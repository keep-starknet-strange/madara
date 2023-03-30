mod overrides;

use std::collections::BTreeMap;
use std::sync::Arc;

use codec::Decode;
use mp_starknet::storage::{StarknetStorageSchema, PALLET_STARKNET_SCHEMA};
pub use overrides::*;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_storage::StorageKey;

pub fn overrides_handle<B, C, BE>(client: Arc<C>) -> Arc<OverrideHandle<B>>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
{
    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        StarknetStorageSchema::V1,
        Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_>>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::<B, C>::new(client)),
    })
}

pub fn onchain_storage_schema<B: BlockT, C, BE>(client: &C, hash: B::Hash) -> StarknetStorageSchema
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
{
    match client.storage(hash, &StorageKey(PALLET_STARKNET_SCHEMA.to_vec())) {
        Ok(Some(bytes)) => Decode::decode(&mut &bytes.0[..]).ok().unwrap_or(StarknetStorageSchema::Undefined),
        _ => StarknetStorageSchema::Undefined,
    }
}
