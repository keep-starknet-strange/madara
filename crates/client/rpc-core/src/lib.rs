//! Starknet RPC API trait and types
//!
//! Starkware maintains [a description of the Starknet API](https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json)
//! using the openRPC specification.
//! This crate uses `jsonrpsee` to define such an API in Rust terms.

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

pub type FieldElement = String;
pub type BlockNumber = u64;
pub type BlockHash = FieldElement;

/// A tag specifying a dynamic reference to a block
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BlockTag {
    #[serde(rename = "latest")]
    Latest,
    #[serde(rename = "pending")]
    Pending,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct BlockHashAndNumber {
    pub block_hash: FieldElement,
    pub block_number: BlockNumber,
}

/// Function call information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FunctionCall {
    pub contract_address: FieldElement,
    pub entry_point_selector: FieldElement,
    pub calldata: Vec<FieldElement>,
}

// In order to mix tagged and untagged {de}serialization for BlockId (see starknet RPC standard)
// in the same object, we need this kind of workaround with intermediate types

#[derive(Serialize, Deserialize)]
enum BlockIdTagged {
    #[serde(rename = "block_hash")]
    BlockHash(BlockHash),
    #[serde(rename = "block_number")]
    BlockNumber(BlockNumber),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum BlockIdUntagged {
    Tagged(BlockIdTagged),
    BlockTag(BlockTag),
}

/// A block hash, number or tag
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(from = "BlockIdUntagged")]
#[serde(into = "BlockIdUntagged")]
pub enum BlockId {
    BlockHash(BlockHash),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
}

impl From<BlockIdUntagged> for BlockId {
    fn from(value: BlockIdUntagged) -> Self {
        match value {
            BlockIdUntagged::Tagged(v) => match v {
                BlockIdTagged::BlockHash(h) => Self::BlockHash(h),
                BlockIdTagged::BlockNumber(n) => Self::BlockNumber(n),
            },
            BlockIdUntagged::BlockTag(t) => Self::BlockTag(t),
        }
    }
}

impl From<BlockId> for BlockIdUntagged {
    fn from(value: BlockId) -> Self {
        match value {
            BlockId::BlockHash(h) => Self::Tagged(BlockIdTagged::BlockHash(h)),
            BlockId::BlockNumber(n) => Self::Tagged(BlockIdTagged::BlockNumber(n)),
            BlockId::BlockTag(t) => Self::BlockTag(t),
        }
    }
}

/// Starknet rpc interface.
#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    /// Get the most recent accepted block number
    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<BlockNumber>;

    /// Get the most recent accepted block hash and number
    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber>;

    /// Get the number of transactions in a block given a block id
    #[method(name = "getBlockTransactionCount")]
    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128>;

    /// Call a contract function at a given block id
    #[method(name = "call")]
    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>>;
}
