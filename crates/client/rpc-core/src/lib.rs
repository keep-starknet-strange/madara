use jsonrpsee::{core::RpcResult as Result, proc_macros::rpc};

use kaioshin_runtime::opaque::Block;

#[rpc(server, namespace = "starknet")]
#[async_trait]
pub trait StarkNetRpc {
    #[method(name = "getBlockWithTxHashes")]
    fn get_block_with_tx_hashes(&self, block_id: u64) -> Result<Option<Block>>;
}
