use mc_rpc_core::utils::get_block_by_block_hash;
use mp_digest_log::{find_starknet_block, FindLogError};
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Backend as _, HeaderBackend};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Zero};

fn sync_block<B: BlockT, C, BE, H>(client: &C, backend: &mc_db::Backend<B>, header: &B::Header) -> anyhow::Result<()>
where
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    BE: Backend<B>,
    H: HasherT,
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
                Ok(storage_starknet_block) => {
                    let digest_starknet_block_hash = digest_starknet_block.header().hash::<H>();
                    let storage_starknet_block_hash = storage_starknet_block.header().hash::<H>();
                    // Ensure the two blocks sources (chain storage and block digest) agree on the block content
                    if digest_starknet_block_hash != storage_starknet_block_hash {
                        Err(anyhow::anyhow!(
                            "Starknet block hash mismatch: madara consensus digest ({digest_starknet_block_hash:?}), \
                             db state ({storage_starknet_block_hash:?})"
                        ))
                    } else {
                        let chain_id = client.runtime_api().chain_id(substrate_block_hash)?;

                        // Success, we write the Starknet to Substate hashes mapping to db
                        let mapping_commitment = mc_db::MappingCommitment {
                            block_hash: substrate_block_hash,
                            starknet_block_hash: digest_starknet_block_hash.into(),
                            starknet_transaction_hashes: digest_starknet_block
                                .transactions()
                                .iter()
                                .map(|tx| tx.compute_hash::<H>(chain_id, false).into())
                                .collect(),
                        };

                        backend.mapping().write_hashes(mapping_commitment).map_err(|e| anyhow::anyhow!(e))
                    }
                }
                // If there is not Starknet block in this Substrate block, we write it in the db
                Err(_) => backend.mapping().write_none(substrate_block_hash).map_err(|e| anyhow::anyhow!(e)),
            }
        }
        // If there is not Starknet block in this Substrate block, we write it in the db
        Err(FindLogError::NotLog) => backend.mapping().write_none(substrate_block_hash).map_err(|e| anyhow::anyhow!(e)),
        Err(FindLogError::MultipleLogs) => Err(anyhow::anyhow!("Multiple logs found")),
    }
}

fn sync_genesis_block<B: BlockT, C, H>(
    _client: &C,
    backend: &mc_db::Backend<B>,
    header: &B::Header,
) -> anyhow::Result<()>
where
    C: HeaderBackend<B>,
    B: BlockT,
    H: HasherT,
{
    let substrate_block_hash = header.hash();

    let block = match find_starknet_block(header.digest()) {
        Ok(block) => block,
        Err(FindLogError::NotLog) => {
            return backend.mapping().write_none(substrate_block_hash).map_err(|e| anyhow::anyhow!(e));
        }
        Err(FindLogError::MultipleLogs) => return Err(anyhow::anyhow!("Multiple logs found")),
    };
    let block_hash = block.header().hash::<H>();
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
) -> anyhow::Result<bool>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT,
{
    let mut current_syncing_tips = madara_backend.meta().current_syncing_tips()?;

    if current_syncing_tips.is_empty() {
        let mut leaves = substrate_backend.blockchain().leaves()?;
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
        sync_genesis_block::<_, _, H>(client, madara_backend, &operating_header)?;

        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    } else {
        sync_block::<_, _, _, H>(client, madara_backend, &operating_header)?;

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
) -> anyhow::Result<bool>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT,
{
    let mut synced_any = false;

    for _ in 0..limit {
        synced_any = synced_any || sync_one_block::<_, _, _, H>(client, substrate_backend, madara_backend, sync_from)?;
    }

    Ok(synced_any)
}

fn fetch_header<B: BlockT, BE>(
    substrate_backend: &BE,
    madara_backend: &mc_db::Backend<B>,
    checking_tip: B::Hash,
    sync_from: <B::Header as HeaderT>::Number,
) -> anyhow::Result<Option<B::Header>>
where
    BE: HeaderBackend<B>,
{
    if madara_backend.mapping().is_synced(&checking_tip)? {
        return Ok(None);
    }

    match substrate_backend.header(checking_tip) {
        Ok(Some(checking_header)) if checking_header.number() >= &sync_from => Ok(Some(checking_header)),
        Ok(Some(_)) => Ok(None),
        Ok(None) | Err(_) => Err(anyhow::anyhow!("Header not found")),
    }
}
