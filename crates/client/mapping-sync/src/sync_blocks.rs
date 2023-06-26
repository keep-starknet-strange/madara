use mc_rpc_core::utils::get_block_by_block_hash;
use mp_digest_log::{find_starknet_block, FindLogError};
use mp_starknet::traits::hash::HasherT;
use mp_starknet::traits::ThreadSafeCopy;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Backend as _, HeaderBackend};
use sp_core::H256;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Zero};

fn sync_block<B: BlockT, C, BE, H>(
    client: &C,
    backend: &mc_db::Backend<B>,
    header: &B::Header,
    hasher: &H,
) -> Result<(), String>
where
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT + ThreadSafeCopy,
{
    // Before storing the new block in the Madara backend database, we want to make sure that the
    // wrapped Starknet block it contains is the same that we can find in the storage at this height.
    // Then we will store the two block hashes (wrapper and wrapped) alongside in our db.

    let substrate_block_hash = header.hash();
    match mp_digest_log::find_starknet_block(header.digest()) {
        Ok(digest_starknet_block) => {
            // Read the runtime storage in order to find the Starknet block stored under this Substrate block
            let opt_storage_starknet_block = get_block_by_block_hash(client, substrate_block_hash);
            match opt_storage_starknet_block {
                Some(storage_starknet_block) => {
                    let digest_starknet_block_hash = digest_starknet_block.header().hash(*hasher);
                    let storage_starknet_block_hash = storage_starknet_block.header().hash(*hasher);
                    // Ensure the two blocks sources (chain storage and block digest) agree on the block content
                    if digest_starknet_block_hash != storage_starknet_block_hash {
                        Err(format!(
                            "Starknet block hash mismatch: madara consensus digest ({digest_starknet_block_hash:?}), \
                             db state ({storage_starknet_block_hash:?})"
                        ))
                    } else {
                        // Success, we write the Starknet to Substate hashes mapping to db
                        let mapping_commitment = mc_db::MappingCommitment {
                            block_hash: substrate_block_hash,
                            starknet_block_hash: digest_starknet_block_hash.into(),
                            starknet_transaction_hashes: digest_starknet_block
                                .transactions()
                                .into_iter()
                                .map(|tx| H256::from(tx.hash))
                                .collect(),
                        };

                        backend.mapping().write_hashes(mapping_commitment)
                    }
                }
                // If there is not Starknet block in this Substrate block, we write it in the db
                None => backend.mapping().write_none(substrate_block_hash),
            }
        }
        // If there is not Starknet block in this Substrate block, we write it in the db
        Err(FindLogError::NotLog) => backend.mapping().write_none(substrate_block_hash),
        Err(FindLogError::MultipleLogs) => Err("Multiple logs found".to_string()),
    }
}

fn sync_genesis_block<B: BlockT, C, H>(
    _client: &C,
    backend: &mc_db::Backend<B>,
    header: &B::Header,
    hasher: &H,
) -> Result<(), String>
where
    C: HeaderBackend<B>,
    B: BlockT,
    H: HasherT + ThreadSafeCopy,
{
    let substrate_block_hash = header.hash();

    let block = match find_starknet_block(header.digest()) {
        Ok(block) => block,
        Err(FindLogError::NotLog) => return backend.mapping().write_none(substrate_block_hash),
        Err(FindLogError::MultipleLogs) => return Err("Multiple logs found".to_string()),
    };
    let block_hash = block.header().hash(*hasher);
    let mapping_commitment = mc_db::MappingCommitment::<B> {
        block_hash: substrate_block_hash,
        starknet_block_hash: block_hash.into(),
        starknet_transaction_hashes: Vec::new(),
    };

    backend.mapping().write_hashes(mapping_commitment)?;

    Ok(())
}

fn sync_one_block<B: BlockT, C, BE, H>(
    client: &C,
    substrate_backend: &BE,
    madara_backend: &mc_db::Backend<B>,
    sync_from: <B::Header as HeaderT>::Number,
    hasher: &H,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT + ThreadSafeCopy,
{
    let mut current_syncing_tips = madara_backend.meta().current_syncing_tips()?;

    if current_syncing_tips.is_empty() {
        let mut leaves = substrate_backend.blockchain().leaves().map_err(|e| format!("{:?}", e))?;
        if leaves.is_empty() {
            return Ok(false);
        }
        current_syncing_tips.append(&mut leaves);
    }

    let mut operating_header = None;
    while let Some(checking_tip) = current_syncing_tips.pop() {
        if let Some(checking_header) =
            fetch_header(substrate_backend.blockchain(), madara_backend, checking_tip, sync_from)?
        {
            operating_header = Some(checking_header);
            break;
        }
    }
    let operating_header = match operating_header {
        Some(operating_header) => operating_header,
        None => {
            madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
            return Ok(false);
        }
    };

    if operating_header.number() == &Zero::zero() {
        sync_genesis_block(client, madara_backend, &operating_header, hasher)?;

        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    } else {
        sync_block(client, madara_backend, &operating_header, hasher)?;

        current_syncing_tips.push(*operating_header.parent_hash());
        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    }
}

pub fn sync_blocks<B: BlockT, C, BE, H>(
    client: &C,
    substrate_backend: &BE,
    madara_backend: &mc_db::Backend<B>,
    limit: usize,
    sync_from: <B::Header as HeaderT>::Number,
    hasher: &H,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT + ThreadSafeCopy,
{
    let mut synced_any = false;

    for _ in 0..limit {
        synced_any = synced_any || sync_one_block(client, substrate_backend, madara_backend, sync_from, hasher)?;
    }

    Ok(synced_any)
}

fn fetch_header<B: BlockT, BE>(
    substrate_backend: &BE,
    madara_backend: &mc_db::Backend<B>,
    checking_tip: B::Hash,
    sync_from: <B::Header as HeaderT>::Number,
) -> Result<Option<B::Header>, String>
where
    BE: HeaderBackend<B>,
{
    if madara_backend.mapping().is_synced(&checking_tip)? {
        return Ok(None);
    }

    match substrate_backend.header(checking_tip) {
        Ok(Some(checking_header)) if checking_header.number() >= &sync_from => Ok(Some(checking_header)),
        Ok(Some(_)) => Ok(None),
        Ok(None) | Err(_) => Err("Header not found".to_string()),
    }
}
