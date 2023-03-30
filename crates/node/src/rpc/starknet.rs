use std::sync::Arc;

use madara_db::Backend;
use sp_api::BlockT;

/// Extra dependencies for Starknet compatibility.
#[derive(Clone)]
pub struct StarknetDeps<C, B: BlockT> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Madara Backend.
    pub madara_backend: Arc<Backend<B>>,
}
