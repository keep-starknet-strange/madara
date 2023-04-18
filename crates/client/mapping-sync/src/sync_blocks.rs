use std::sync::Arc;

use mc_storage::OverrideHandle;
use mp_digest_log::{FindLogError, Hashes, Log, PostLog};
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
    let substrate_block_hash = header.hash();
    match mp_digest_log::find_log(header.digest()) {
        Ok(log) => {
            let gen_from_hashes = |hashes: Hashes| -> mc_db::MappingCommitment<B> {
                mc_db::MappingCommitment { block_hash: substrate_block_hash, starknet_block_hash: hashes.block_hash }
            };
            let gen_from_block = |block| -> mc_db::MappingCommitment<B> {
                let hashes = Hashes::from_block(block);
                gen_from_hashes(hashes)
            };

            match log {
                Log::Post(post_log) => match post_log {
                    PostLog::BlockHash(expect_starknet_block_hash) => {
                        let starknet_block =
                            overrides.for_block_hash(client, substrate_block_hash).current_block(substrate_block_hash);
                        match starknet_block {
                            Some(block) => {
                                let got_starknet_block_hash = block.header().hash();
                                if got_starknet_block_hash != expect_starknet_block_hash {
                                    Err(format!(
                                        "Starknet block hash mismatch: frontier consensus digest \
                                         ({expect_starknet_block_hash:?}), db state ({got_starknet_block_hash:?})"
                                    ))
                                } else {
                                    let mapping_commitment = gen_from_block(block);
                                    backend.mapping().write_hashes(mapping_commitment)
                                }
                            }
                            None => backend.mapping().write_none(substrate_block_hash),
                        }
                    }
                },
            }
        }
        Err(FindLogError::NotFound) => backend.mapping().write_none(substrate_block_hash),
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
