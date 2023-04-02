//! Starknet RPC API trait and types
//!
//! Starkware maintains (a description of Starknet API)[https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json] using the openRPC specification.
//! This crate uses `jsonrpsee` to define such an API in Rust terms.

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

pub type FieldElement = String;
pub type BlockNumber = u64;
pub type BlockHash = FieldElement;

/// A tag specifying a dynamic reference to a blocl
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

/// A block hash, number or tag
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum BlockId {
    BlockHash(FieldElement),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
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
}
