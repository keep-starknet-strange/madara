use std::sync::Arc;

use madara_db::Backend;
use sp_api::BlockT;

/// Extra dependencies for Starknet compatibility.
pub struct StarknetDeps<C, B: BlockT> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Madara Backend.
    pub madara_backend: Arc<Backend<B>>,
}

impl<C, B: BlockT> Clone for StarknetDeps<C, B> {
    fn clone(&self) -> Self {
        Self { client: self.client.clone(), madara_backend: self.madara_backend.clone() }
    }
}
