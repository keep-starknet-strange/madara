use std::sync::Arc;

use mc_db::Backend;
use mc_genesis_data_provider::GenesisProvider;
use mc_storage::OverrideHandle;
use sc_network_sync::SyncingService;
use sp_api::BlockT;
use sp_runtime::traits::Header as HeaderT;

/// Extra dependencies for Starknet compatibility.
pub struct StarknetDeps<C, G: GenesisProvider, B: BlockT> {
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
    /// The genesis state data provider
    pub genesis_provider: Arc<G>,
}

impl<C, G: GenesisProvider, B: BlockT> Clone for StarknetDeps<C, G, B> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            madara_backend: self.madara_backend.clone(),
            overrides: self.overrides.clone(),
            sync_service: self.sync_service.clone(),
            starting_block: self.starting_block,
            genesis_provider: self.genesis_provider.clone(),
        }
    }
}
