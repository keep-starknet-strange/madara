//! Defines a generic implementation of a Starknet JSON-RPC server client.

use starknet_core::types::BlockId;
use starknet_core::types::requests::GetBlockWithTxHashesRequest;

use self::json_rpc::{JsonRpcClient, JsonRpcClientError};

pub mod json_rpc;

// For some reason, `starknet-core` does not define a type that requests the spec version.
// Let's do it ourselves.
struct SpecVersionRequest;

impl self::json_rpc::Request for SpecVersionRequest {
    const METHOD: &'static str = "starknet_specVersion";
    type Response = Box<str>;
    type Params = [(); 0];
    #[inline(always)]
    fn into_params(self) -> Self::Params {
        [(); 0]
    }
}

/// A generic implementation of a Starknet JSON-RPC client.
pub struct StarknetClient<T> {
    inner: JsonRpcClient<T>,
}

impl<T> StarknetClient<T> {
    /// Creates a new [`StarknetClient`] with the given transport layer.
    pub fn new(transport: T) -> Self {
        Self {
            inner: JsonRpcClient::new(transport),
        }
    }
}

impl<T: json_rpc::Transport> StarknetClient<T> {
    /// Requets the version of the Starknet JSON-RPC specification being used by the server.
    pub async fn spec_version(&self) -> Result<Box<str>, JsonRpcClientError<T::Error>> {
        self.inner.request(SpecVersionRequest).await
    }

    /// Returns information about the block with the given ID.
    /// 
    /// # Arguments
    /// 
    /// * `block_id` - An identifier for the queried block.
    /// 
    /// # Returns
    /// 
    /// Information about the block.
    /// 
    /// Transactions are not fully sent in the response. Instead, only their hashes are communicated.
    pub async fn get_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> Result<starknet_core::types::MaybePendingBlockWithTxHashes, JsonRpcClientError<T::Error>> {
        self.inner.request(GetBlockWithTxHashesRequest { block_id }).await
    }
}