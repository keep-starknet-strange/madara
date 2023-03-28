use jsonrpsee::core::Error;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use sp_core::{H256, U256};

pub type BlockNumber = U256;

pub type BlockHash = H256;

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockHashAndNumber {
    pub block_number: BlockNumber,
    pub block_hash: BlockHash,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Tag {
    #[serde(rename = "latest")]
    Latest,

    #[serde(rename = "pending")]
    Pending,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlockHashOrNumber {
    #[serde(rename = "block_hash")]
    Hash(BlockHash),
    #[serde(rename = "block_number")]
    Number(BlockNumber),
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BlockId {
    HashOrNumber(BlockHashOrNumber),
    Tag(Tag),
}

#[rpc(server, client, namespace = "starknet")]
pub trait StarknetRpcApi {
    #[method(name = "blockNumber")]
    fn block_number(&self) -> Result<BlockNumber, Error>;

    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, Error>;
}
