use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use mp_starknet::starknet_block::block::Block;
use mp_starknet::storage::StarknetStorageSchema;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
// Substrate
use sp_api::ProvideRuntimeApi;
use sp_io::hashing::twox_128;
use sp_runtime::traits::Block as BlockT;

mod schema_v1_override;

pub use self::schema_v1_override::SchemaV1Override;

pub struct OverrideHandle<Block: BlockT> {
    pub schemas: BTreeMap<StarknetStorageSchema, Box<dyn StorageOverride<Block>>>,
    pub fallback: Box<dyn StorageOverride<Block>>,
}

/// Something that can fetch Starknet-related data. This trait is quite similar to the runtime API,
/// and indeed the implementation of it uses the runtime API.
/// Having this trait is useful because it allows optimized implementations that fetch data from a
/// State Backend with some assumptions about pallet-ethereum's storage schema. Using such an
/// optimized implementation avoids spawning a runtime and the overhead associated with it.
pub trait StorageOverride<B: BlockT>: Send + Sync {
    /// Return the current block.
    fn current_block(&self, block_hash: B::Hash) -> Option<Block>;
}

fn storage_prefix_build(module: &[u8], storage: &[u8]) -> Vec<u8> {
    [twox_128(module), twox_128(storage)].concat().to_vec()
}

/// A wrapper type for the Runtime API. This type implements `StorageOverride`, so it can be used
/// when calling the runtime API is desired but a `dyn StorageOverride` is required.
pub struct RuntimeApiStorageOverride<B: BlockT, C> {
    client: Arc<C>,
    _marker: PhantomData<B>,
}

impl<B: BlockT, C> RuntimeApiStorageOverride<B, C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: PhantomData }
    }
}

impl<B, C> StorageOverride<B> for RuntimeApiStorageOverride<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + Send + Sync,
    C::Api: StarknetRuntimeApi<B>,
{
    /// Return the current block.
    fn current_block(&self, block_hash: B::Hash) -> Option<Block> {
        let api = self.client.runtime_api();

        api.current_block(block_hash).ok()
    }
}
