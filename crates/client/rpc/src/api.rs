use jsonrpsee::{core::Error, proc_macros::rpc};

use mp_starknet::starknet_block::block::Block;
use sp_core::H256;

#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    #[method(name = "blockNumber")]
    fn block_hash(&self) -> Result<Option<H256>, Error>;

    // #[method(name = "getBlockWithTxHashes")]
    // fn get_block_with_tx_hashes(&self, block_id: u64) -> Result<Block, Error>;
}
