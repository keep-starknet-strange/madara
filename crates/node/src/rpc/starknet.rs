use std::sync::Arc;

use mc_db::Backend;
use mc_rpc::cache::StarknetDataCacheTask;
use mc_storage::OverrideHandle;
use sc_network_sync::SyncingService;
use sp_api::BlockT;
use sp_runtime::traits::Header as HeaderT;

/// Extra dependencies for Starknet compatibility.
pub struct StarknetDeps<C, B: BlockT> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Madara Backend.
    pub madara_backend: Arc<Backend<B>>,
    /// Starknet data access overrides.
    pub overrides: Arc<OverrideHandle<B>>,
    /// The Substrate client sync service.
    pub sync_service: Arc<SyncingService<B>>,
    /// The starting block for the syncing.
    pub starting_block: <<B>::Header as HeaderT>::Number,
    /// Cache for Starknet data
    pub data_cache: Arc<StarknetDataCacheTask<B>>,
}

impl<C, B: BlockT> Clone for StarknetDeps<C, B> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            madara_backend: self.madara_backend.clone(),
            overrides: self.overrides.clone(),
            sync_service: self.sync_service.clone(),
            starting_block: self.starting_block,
            data_cache: self.data_cache.clone(),
        }
    }
}
