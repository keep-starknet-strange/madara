//! Starknet RPC API trait and types
//!
//! Starkware maintains [a description of the Starknet API](https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json)
//! using the openRPC specification.
//! This crate uses `jsonrpsee` to define such an API in Rust terms.

#[cfg(test)]
mod tests;

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use types::{BlockHashAndNumber, BlockId, BlockNumber, FunctionCall};

pub mod utils;

/// Starknet rpc interface.
#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber>;

    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<BlockNumber>;

    #[method(name = "call")]
    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<FieldElement>;

    #[method(name = "chainId")]
    fn chain_id(&self) -> RpcResult<ChainId> {
        todo!("Not implemented");
    }

    #[method(name = "estimateFee")]
    fn estimate_fee(&self, request: BroadcastedTransaction, block_id: BlockId) -> RpcResult<Estimation> {
        todo!("Not implemented");
    }

    #[method(name = "getBlockWithTxHashes")]
    fn get_block_with_tx_hashes(&self, block_id: BlockId) -> RpcResult<StarknetGetBlockHashWithTxHashesResult> {
        todo!("Not implemented");
    }

    #[method(name = "getBlockWithTxs")]
    fn get_block_with_txs(&self, block_id: BlockId) -> RpcResult<StarknetGetBlockWithTxsResult> {
        todo!("Not implemented");
    }

    #[method(name = "getClass")]
    fn get_class(&self, block_id: BlockId, class_hash: FieldElement) -> RpcResult<RPCContractClass>;

    #[method(name = "getClassAt")]
    fn get_class_at(&self, block_id: BlockId, contract_address: Address) -> RpcResult<RPCContractClass>;

    #[method(name = "getClassHashAt")]
    fn get_class_hash_at(&self, block_id: BlockId, contract_address: Address) -> RpcResult<FieldElement>;

    #[method(name = "getBlockTransactionCount")]
    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<BlockTransactionCount>;

    #[method(name = "getEvents")]
    fn get_events(&self, filter: EventEmitter) -> RpcResult<EventsChunk> {
        todo!("Not implemented");
    }

    #[method(name = "getNonce")]
    fn get_nonce(&self, block_id: BlockId, contract_address: Address) -> RpcResult<FieldElement> {
        todo!("Not implemented");
    }

    #[method(name = "getStateUpdate")]
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<StarknetGetStateUpdateResult> {
        todo!("Not implemented");
    }

    #[method(name = "getStorageAt")]
    fn get_storage_at(&self, contract_address: Address, key: StorageKey, block_id: BlockId) -> RpcResult<FieldElement> {
        todo!("Not implemented");
    }

    #[method(name = "getTransactionByHash")]
    fn get_transaction_by_hash(&self, transaction_hash: TransactionHash) -> RpcResult<Transaction> {
        todo!("Not implemented");
    }

    #[method(name = "getTransactionByBlockIdAndIndex")]
    fn get_transaction_by_block_id_and_index(&self, block_id: BlockId, index: Index) -> RpcResult<Transaction> {
        todo!("Not implemented");
    }

    #[method(name = "getTransactionReceipt")]
    fn get_transaction_receipt(&self, transaction_hash: TransactionHash) -> RpcResult<TransactionReceipt> {
        todo!("Not implemented");
    }

    #[method(name = "pendingTransactions")]
    fn pending_transactions(&self) -> RpcResult<PendingTransactions> {
        todo!("Not implemented");
    }

    #[method(name = "syncing")]
    fn syncing(&self) -> RpcResult<SyncingStatus>;
}
