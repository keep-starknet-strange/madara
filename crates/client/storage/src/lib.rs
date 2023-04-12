//! Storage overrides readers
//!
//! In order for the client to access on pallets chain data data it has to read from the storage.
//! This can be achieve either through the pallet runtime API or by indexing the storage directly.
//! The `OverrideHandle` make it possible to use the later, more efficient way, while keeping the
//! first one as a fallback.
//! It can also support multiple versions of the pallet storage.

mod overrides;

use std::collections::BTreeMap;
use std::sync::Arc;

use mp_starknet::storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
pub use overrides::*;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use scale_codec::Decode;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_storage::StorageKey;

/// Create and return a handle of the starknet schema overrides
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
        StarknetStorageSchemaVersion::V1,
        Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_>>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::<B, C>::new(client)),
    })
}

/// Retrieve the current `pallet-starknet` storage schema version
pub fn onchain_storage_schema<B, C, BE>(client: &C, hash: B::Hash) -> StarknetStorageSchemaVersion
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
{
    match client.storage(hash, &StorageKey(PALLET_STARKNET_SCHEMA.to_vec())) {
        Ok(Some(bytes)) => Decode::decode(&mut &bytes.0[..]).ok().unwrap_or(StarknetStorageSchemaVersion::Undefined),
        _ => StarknetStorageSchemaVersion::Undefined,
    }
}
