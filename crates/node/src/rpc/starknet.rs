use std::sync::Arc;

use mc_db::Backend;
use mc_storage::OverrideHandle;
use sp_api::BlockT;

/// Extra dependencies for Starknet compatibility.
pub struct StarknetDeps<C, B: BlockT> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Madara Backend.
    pub madara_backend: Arc<Backend<B>>,
    /// Starknet data access overrides.
    pub overrides: Arc<OverrideHandle<B>>,
}

impl<C, B: BlockT> Clone for StarknetDeps<C, B> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            madara_backend: self.madara_backend.clone(),
            overrides: self.overrides.clone(),
        }
    }
}
