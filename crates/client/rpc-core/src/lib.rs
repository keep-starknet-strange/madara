use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

pub type BlockNumber = u64;
pub type FieldElement = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct BlockHashAndNumber {
    pub block_hash: FieldElement,
    pub block_number: BlockNumber,
}

/// Starkware rpc interface.
#[rpc(server, namespace = "starknet")]
#[async_trait]
pub trait StarknetRpcApi {
    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<BlockNumber>;

    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber>;
}
