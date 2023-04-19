use std::sync::Arc;

use mc_storage::OverrideHandle;
use mp_digest_log::FindLogError;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Backend as _, HeaderBackend};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Zero};

fn sync_block<B: BlockT, C, BE>(
    client: &C,
    overrides: Arc<OverrideHandle<B>>,
    backend: &mc_db::Backend<B>,
    header: &B::Header,
) -> Result<(), String>
where
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
{
    // Before storing the new block in the Madara backend database, we want to make sure that the
    // wrapped Starknet block it contains is the same that we can find in the storage at this height.
    // Then we will store the two block hashes (wrapper and wrapped) alongside in our db.

    let substrate_block_hash = header.hash();
    match mp_digest_log::find_starknet_block(header.digest()) {
        Ok(digest_starknet_block) => {
            // Read the runtime storage in order to find the Starknet block stored under this Substrate block
            let opt_storage_starknet_block =
                overrides.for_block_hash(client, substrate_block_hash).current_block(substrate_block_hash);
            match opt_storage_starknet_block {
                Some(storage_starknet_block) => {
                    let digest_starknet_block_hash = digest_starknet_block.header().hash();
                    let storage_starknet_block_hash = storage_starknet_block.header().hash();
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
                            starknet_block_hash: digest_starknet_block_hash,
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

fn sync_genesis_block<B: BlockT, C>(client: &C, backend: &mc_db::Backend<B>, header: &B::Header) -> Result<(), String>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    let substrate_block_hash = header.hash();

    let block = client.runtime_api().current_block(substrate_block_hash).map_err(|e| format!("{:?}", e))?;
    let block_hash = block.header().hash();
    let mapping_commitment =
        mc_db::MappingCommitment::<B> { block_hash: substrate_block_hash, starknet_block_hash: block_hash };
    backend.mapping().write_hashes(mapping_commitment)?;

    Ok(())
}

fn sync_one_block<B: BlockT, C, BE>(
    client: &C,
    substrate_backend: &BE,
    overrides: Arc<OverrideHandle<B>>,
    madara_backend: &mc_db::Backend<B>,
    sync_from: <B::Header as HeaderT>::Number,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
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
        sync_genesis_block(client, madara_backend, &operating_header)?;

        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    } else {
        sync_block(client, overrides, madara_backend, &operating_header)?;

        current_syncing_tips.push(*operating_header.parent_hash());
        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    }
}

pub fn sync_blocks<B: BlockT, C, BE>(
    client: &C,
    substrate_backend: &BE,
    overrides: Arc<OverrideHandle<B>>,
    madara_backend: &mc_db::Backend<B>,
    limit: usize,
    sync_from: <B::Header as HeaderT>::Number,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
{
    let mut synced_any = false;

    for _ in 0..limit {
        synced_any =
            synced_any || sync_one_block(client, substrate_backend, overrides.clone(), madara_backend, sync_from)?;
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
