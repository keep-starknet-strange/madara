use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;

pub fn load_hash<B: BlockT, C>(client: &C, backend: &mc_db::Backend<B>, hash: H256) -> Result<Option<B::Hash>, String>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    let substrate_hashes = backend.mapping().block_hash(&hash)?;

    if let Some(substrate_hashes) = substrate_hashes {
        for substrate_hash in substrate_hashes {
            if is_canon::<B, C>(client, substrate_hash) {
                return Ok(Some(substrate_hash));
            }
        }
    }
    Ok(None)
}

pub fn is_canon<B: BlockT, C>(client: &C, target_hash: B::Hash) -> bool
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    if let Ok(Some(number)) = client.number(target_hash) {
        if let Ok(Some(hash)) = client.hash(number) {
            return hash == target_hash;
        }
    }
    false
}
