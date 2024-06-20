use std::sync::Arc;

use jsonrpsee::core::{async_trait, RpcResult};
use mc_genesis_data_provider::GenesisProvider;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mp_hashers::HasherT;
use mp_transactions::{BroadcastedDeclareTransactionV0, TransactionStatus};
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_block_builder::GetPendingBlockExtrinsics;
use sc_client_api::backend::{Backend, StorageProvider};
use sc_client_api::BlockBackend;
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::{
    BlockHashAndNumber, BlockId, BroadcastedDeclareTransaction, BroadcastedDeployAccountTransaction,
    BroadcastedInvokeTransaction, BroadcastedTransaction, ContractClass, DeclareTransactionResult,
    DeployAccountTransactionResult, EventFilterWithPage, EventsPage, FeeEstimate, FieldElement, FunctionCall,
    InvokeTransactionResult, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
    MaybePendingTransactionReceipt, MsgFromL1, SimulatedTransaction, SimulationFlag, SimulationFlagForEstimateFee,
    SyncStatusType, Transaction, TransactionTrace, TransactionTraceWithHash,
};

use crate::Starknet;

// Newtype Wrapper to escape Arc orphan rules
pub struct StarknetRpcWrapper<A: ChainApi, B: BlockT, BE, G, C, P, H>(pub Arc<Starknet<A, B, BE, G, C, P, H>>);

impl<A: ChainApi, B: BlockT, BE, G, C, P, H> Clone for StarknetRpcWrapper<A, B, BE, G, C, P, H> {
    fn clone(&self) -> Self {
        StarknetRpcWrapper(self.0.clone())
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> MadaraRpcApiServer for StarknetRpcWrapper<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    G: GenesisProvider + Send + Sync + 'static,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    H: HasherT + Send + Sync + 'static,
{
    fn predeployed_accounts(&self) -> RpcResult<Vec<PredeployedAccountWithBalance>> {
        self.0.predeployed_accounts()
    }

    async fn add_declare_transaction_v0(
        &self,
        params: BroadcastedDeclareTransactionV0,
    ) -> RpcResult<DeclareTransactionResult> {
        self.0.add_declare_transaction_v0(params).await
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetReadRpcApiServer for StarknetRpcWrapper<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    G: GenesisProvider + Send + Sync + 'static,
    H: HasherT + Send + Sync + 'static,
{
    /// Returns the Version of the StarkNet JSON-RPC Specification Being Used
    ///
    /// This method provides the version of the StarkNet JSON-RPC specification that the node is
    /// currently using. The version is returned as a semantic versioning (SemVer) string.
    ///
    /// # Arguments
    ///
    /// This method does not take any arguments.
    ///
    /// # Returns
    ///
    /// * `spec_version` - A string representing the SemVer of the StarkNet JSON-RPC specification
    ///   being used.
    fn spec_version(&self) -> RpcResult<String> {
        self.0.spec_version()
    }

    /// Get the Most Recent Accepted Block Number
    ///
    /// ### Arguments
    ///
    /// This function does not take any arguments.
    ///
    /// ### Returns
    ///
    /// * `block_number` - The latest block number of the current network.
    fn block_number(&self) -> RpcResult<u64> {
        self.0.block_number()
    }

    /// Get the Most Recent Accepted Block Hash and Number
    ///
    /// ### Arguments
    ///
    /// This function does not take any arguments.
    ///
    /// ### Returns
    ///
    /// * `block_hash_and_number` - A tuple containing the latest block hash and number of the
    ///   current network.
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber> {
        self.0.block_hash_and_number()
    }

    /// Get the Number of Transactions in a Given Block
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The identifier of the requested block. This can be the hash of the block, the
    ///   block's number (height), or a specific block tag.
    ///
    /// ### Returns
    ///
    /// * `transaction_count` - The number of transactions in the specified block.
    ///
    /// ### Errors
    ///
    /// This function may return a `BLOCK_NOT_FOUND` error if the specified block does not exist in
    /// the blockchain.
    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128> {
        self.0.get_block_transaction_count(block_id)
    }

    /// Gets the Transaction Status, Including Mempool Status and Execution Details
    ///
    /// This method retrieves the status of a specified transaction. It provides information on
    /// whether the transaction is still in the mempool, has been executed, or dropped from the
    /// mempool. The status includes both finality status and execution status of the
    /// transaction.
    ///
    /// ### Arguments
    ///
    /// * `transaction_hash` - The hash of the transaction for which the status is requested.
    ///
    /// ### Returns
    ///
    /// * `transaction_status` - An object containing the transaction status details:
    ///   - `finality_status`: The finality status of the transaction, indicating whether it is
    ///     confirmed, pending, or rejected.
    ///   - `execution_status`: The execution status of the transaction, providing details on the
    ///     execution outcome if the transaction has been processed.
    fn get_transaction_status(&self, transaction_hash: FieldElement) -> RpcResult<TransactionStatus> {
        self.0.get_transaction_status(transaction_hash)
    }

    /// Get the value of the storage at the given address and key.
    ///
    /// This function retrieves the value stored in a specified contract's storage, identified by a
    /// contract address and a storage key, within a specified block in the current network.
    ///
    /// ### Arguments
    ///
    /// * `contract_address` - The address of the contract to read from. This parameter identifies
    ///   the contract whose storage is being queried.
    /// * `key` - The key to the storage value for the given contract. This parameter specifies the
    ///   particular storage slot to be queried.
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag. This parameter defines the state of the blockchain at which the storage
    ///   value is to be read.
    ///
    /// ### Returns
    ///
    /// Returns the value at the given key for the given contract, represented as a `FieldElement`.
    /// If no value is found at the specified storage key, returns 0.
    ///
    /// ### Errors
    ///
    /// This function may return errors in the following cases:
    ///
    /// * `BLOCK_NOT_FOUND` - If the specified block does not exist in the blockchain.
    /// * `CONTRACT_NOT_FOUND` - If the specified contract does not exist or is not deployed at the
    ///   given `contract_address` in the specified block.
    /// * `STORAGE_KEY_NOT_FOUND` - If the specified storage key does not exist within the given
    ///   contract.
    fn get_storage_at(&self, contract_address: FieldElement, key: FieldElement, block_id: BlockId) -> RpcResult<Felt> {
        self.0.get_storage_at(contract_address, key, block_id)
    }

    /// Call a Function in a Contract Without Creating a Transaction
    ///
    /// ### Arguments
    ///
    /// * `request` - The details of the function call to be made. This includes information such as
    ///   the contract address, function signature, and arguments.
    /// * `block_id` - The identifier of the block used to reference the state or call the
    ///   transaction on. This can be the hash of the block, its number (height), or a specific
    ///   block tag.
    ///
    /// ### Returns
    ///
    /// * `result` - The function's return value, as defined in the Cairo output. This is an array
    ///   of field elements (`Felt`).
    ///
    /// ### Errors
    ///
    /// This method may return the following errors:
    /// * `CONTRACT_NOT_FOUND` - If the specified contract address does not exist.
    /// * `CONTRACT_ERROR` - If there is an error with the contract or the function call.
    /// * `BLOCK_NOT_FOUND` - If the specified block does not exist in the blockchain.
    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>> {
        self.0.call(request, block_id)
    }

    /// Get the Contract Class Definition at a Given Address in a Specific Block
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The identifier of the block. This can be the hash of the block, its number
    ///   (height), or a specific block tag.
    /// * `contract_address` - The address of the contract whose class definition will be returned.
    ///
    /// ### Returns
    ///
    /// * `contract_class` - The contract class definition. This may be either a standard contract
    ///   class or a deprecated contract class, depending on the contract's status and the
    ///   blockchain's version.
    ///
    /// ### Errors
    ///
    /// This method may return the following errors:
    /// * `BLOCK_NOT_FOUND` - If the specified block does not exist in the blockchain.
    /// * `CONTRACT_NOT_FOUND` - If the specified contract address does not exist.
    fn get_class_at(&self, block_id: BlockId, contract_address: FieldElement) -> RpcResult<ContractClass> {
        self.0.get_class_at(block_id, contract_address)
    }

    /// Get the contract class hash in the given block for the contract deployed at the given
    /// address
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag
    /// * `contract_address` - The address of the contract whose class hash will be returned
    ///
    /// ### Returns
    ///
    /// * `class_hash` - The class hash of the given contract
    fn get_class_hash_at(&self, block_id: BlockId, contract_address: FieldElement) -> RpcResult<Felt> {
        self.0.get_class_hash_at(block_id, contract_address)
    }

    /// Returns an object about the sync status, or false if the node is not synching
    ///
    /// ### Arguments
    ///
    /// This function does not take any arguments.
    ///
    /// ### Returns
    ///
    /// * `Syncing` - An Enum that can either be a `mc_rpc_core::SyncStatus` struct representing the
    ///   sync status, or a `Boolean` (`false`) indicating that the node is not currently
    ///   synchronizing.
    ///
    /// This is an asynchronous function due to its reliance on `sync_service.best_seen_block()`,
    /// which potentially involves network communication and processing to determine the best block
    /// seen by the sync service.
    async fn syncing(&self) -> RpcResult<SyncStatusType> {
        self.0.syncing().await
    }

    /// Get the contract class definition in the given block associated with the given hash.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag.
    /// * `class_hash` - The hash of the requested contract class.
    ///
    /// ### Returns
    ///
    /// Returns the contract class definition if found. In case of an error, returns a
    /// `StarknetRpcApiError` indicating either `BlockNotFound` or `ClassHashNotFound`.
    fn get_class(&self, block_id: BlockId, class_hash: FieldElement) -> RpcResult<ContractClass> {
        self.0.get_class(block_id, class_hash)
    }

    /// Get block information with transaction hashes given the block id.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag.
    ///
    /// ### Returns
    ///
    /// Returns block information with transaction hashes. This includes either a confirmed block or
    /// a pending block with transaction hashes, depending on the state of the requested block.
    /// In case the block is not found, returns a `StarknetRpcApiError` with `BlockNotFound`.
    fn get_block_with_tx_hashes(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxHashes> {
        self.0.get_block_with_tx_hashes(block_id)
    }

    /// Get the nonce associated with the given address in the given block.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag. This parameter specifies the block in which the nonce is to be checked.
    /// * `contract_address` - The address of the contract whose nonce we're seeking. This is the
    ///   unique identifier of the contract in the Starknet network.
    ///
    /// ### Returns
    ///
    /// Returns the contract's nonce at the requested state. The nonce is returned as a
    /// `FieldElement`, representing the current state of the contract in terms of transactions
    /// count or other contract-specific operations. In case of errors, such as
    /// `BLOCK_NOT_FOUND` or `CONTRACT_NOT_FOUND`, returns a `StarknetRpcApiError` indicating the
    /// specific issue.
    fn get_nonce(&self, block_id: BlockId, contract_address: FieldElement) -> RpcResult<Felt> {
        self.0.get_nonce(block_id, contract_address)
    }

    /// Return the currently configured chain id.
    ///
    /// This function provides the chain id for the network that the node is connected to. The chain
    /// id is a unique identifier that distinguishes between different networks, such as mainnet or
    /// testnet.
    ///
    /// ### Arguments
    ///
    /// This function does not take any arguments.
    ///
    /// ### Returns
    ///
    /// Returns the chain id this node is connected to. The chain id is returned as a specific type,
    /// defined by the Starknet protocol, indicating the particular network.
    fn chain_id(&self) -> RpcResult<Felt> {
        self.0.chain_id()
    }

    /// Estimate the fee associated with transaction
    ///
    /// # Arguments
    ///
    /// * `request` - starknet transaction request
    /// * `block_id` - hash of the requested block, number (height), or tag
    ///
    /// # Returns
    ///
    /// * `fee_estimate` - fee estimate in gwei
    async fn estimate_fee(
        &self,
        request: Vec<BroadcastedTransaction>,
        simulation_flags: Vec<SimulationFlagForEstimateFee>,
        block_id: BlockId,
    ) -> RpcResult<Vec<FeeEstimate>> {
        StarknetReadRpcApiServer::estimate_fee(&*self.0, request, simulation_flags, block_id).await
    }

    /// Estimate the L2 fee of a message sent on L1
    ///
    /// # Arguments
    ///
    /// * `message` - the message to estimate
    /// * `block_id` - hash, number (height), or tag of the requested block
    ///
    /// # Returns
    ///
    /// * `FeeEstimate` - the fee estimation (gas consumed, gas price, overall fee, unit)
    ///
    /// # Errors
    ///
    /// BlockNotFound : If the specified block does not exist.
    /// ContractNotFound : If the specified contract address does not exist.
    /// ContractError : If there is an error with the contract.
    async fn estimate_message_fee(&self, message: MsgFromL1, block_id: BlockId) -> RpcResult<FeeEstimate> {
        self.0.estimate_message_fee(message, block_id).await
    }

    /// Get the details of a transaction by a given block id and index.
    ///
    /// This function fetches the details of a specific transaction in the StarkNet network by
    /// identifying it through its block and position (index) within that block. If no transaction
    /// is found at the specified index, null is returned.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag. This parameter is used to specify the block in which the transaction is
    ///   located.
    /// * `index` - An integer representing the index in the block where the transaction is expected
    ///   to be found. The index starts from 0 and increases sequentially for each transaction in
    ///   the block.
    ///
    /// ### Returns
    ///
    /// Returns the details of the transaction if found, including the transaction hash. The
    /// transaction details are returned as a type conforming to the StarkNet protocol. In case of
    /// errors like `BLOCK_NOT_FOUND` or `INVALID_TXN_INDEX`, returns a `StarknetRpcApiError`
    /// indicating the specific issue.
    fn get_transaction_by_block_id_and_index(&self, block_id: BlockId, index: u64) -> RpcResult<Transaction> {
        self.0.get_transaction_by_block_id_and_index(block_id, index)
    }

    /// Get block information with full transactions given the block id.
    ///
    /// This function retrieves detailed information about a specific block in the StarkNet network,
    /// including all transactions contained within that block. The block is identified using its
    /// unique block id, which can be the block's hash, its number (height), or a block tag.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag. This parameter is used to specify the block from which to retrieve
    ///   information and transactions.
    ///
    /// ### Returns
    ///
    /// Returns detailed block information along with full transactions. Depending on the state of
    /// the block, this can include either a confirmed block or a pending block with its
    /// transactions. In case the specified block is not found, returns a `StarknetRpcApiError` with
    /// `BlockNotFound`.
    fn get_block_with_txs(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxs> {
        self.0.get_block_with_txs(block_id)
    }

    /// Get the information about the result of executing the requested block.
    ///
    /// This function fetches details about the state update resulting from executing a specific
    /// block in the StarkNet network. The block is identified using its unique block id, which can
    /// be the block's hash, its number (height), or a block tag.
    ///
    /// ### Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag. This parameter specifies the block for which the state update information
    ///   is required.
    ///
    /// ### Returns
    ///
    /// Returns information about the state update of the requested block, including any changes to
    /// the state of the network as a result of the block's execution. This can include a confirmed
    /// state update or a pending state update. If the block is not found, returns a
    /// `StarknetRpcApiError` with `BlockNotFound`.
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<MaybePendingStateUpdate> {
        self.0.get_state_update(block_id)
    }

    /// Returns all events matching the given filter.
    ///
    /// This function retrieves all event objects that match the conditions specified in the
    /// provided event filter. The filter can include various criteria such as contract addresses,
    /// event types, and block ranges. The function supports pagination through the result page
    /// request schema.
    ///
    /// ### Arguments
    ///
    /// * `filter` - The conditions used to filter the returned events. The filter is a combination
    ///   of an event filter and a result page request, allowing for precise control over which
    ///   events are returned and in what quantity.
    ///
    /// ### Returns
    ///
    /// Returns a chunk of event objects that match the filter criteria, encapsulated in an
    /// `EventsChunk` type. The chunk includes details about the events, such as their data, the
    /// block in which they occurred, and the transaction that triggered them. In case of
    /// errors, such as `PAGE_SIZE_TOO_BIG`, `INVALID_CONTINUATION_TOKEN`, `BLOCK_NOT_FOUND`, or
    /// `TOO_MANY_KEYS_IN_FILTER`, returns a `StarknetRpcApiError` indicating the specific issue.
    async fn get_events(&self, filter: EventFilterWithPage) -> RpcResult<EventsPage> {
        self.0.get_events(filter).await
    }

    /// Get the details and status of a submitted transaction.
    ///
    /// This function retrieves the detailed information and status of a transaction identified by
    /// its hash. The transaction hash uniquely identifies a specific transaction that has been
    /// submitted to the StarkNet network.
    ///
    /// ### Arguments
    ///
    /// * `transaction_hash` - The hash of the requested transaction. This parameter specifies the
    ///   transaction for which details and status are requested.
    ///
    /// ### Returns
    ///
    /// Returns information about the requested transaction, including its status, sender,
    /// recipient, and other transaction details. The information is encapsulated in a `Transaction`
    /// type, which is a combination of the `TXN` schema and additional properties, such as the
    /// `transaction_hash`. In case the specified transaction hash is not found, returns a
    /// `StarknetRpcApiError` with `TXN_HASH_NOT_FOUND`.
    ///
    /// ### Errors
    ///
    /// The function may return one of the following errors if encountered:
    /// - `PAGE_SIZE_TOO_BIG` if the requested page size exceeds the allowed limit.
    /// - `INVALID_CONTINUATION_TOKEN` if the provided continuation token is invalid or expired.
    /// - `BLOCK_NOT_FOUND` if the specified block is not found.
    /// - `TOO_MANY_KEYS_IN_FILTER` if there are too many keys in the filter, which may exceed the
    ///   system's capacity.
    fn get_transaction_by_hash(&self, transaction_hash: FieldElement) -> RpcResult<Transaction> {
        self.0.get_transaction_by_hash(transaction_hash)
    }

    /// Get the transaction receipt by the transaction hash.
    ///
    /// This function retrieves the transaction receipt for a specific transaction identified by its
    /// hash. The transaction receipt includes information about the execution status of the
    /// transaction, events generated during its execution, and other relevant details.
    ///
    /// ### Arguments
    ///
    /// * `transaction_hash` - The hash of the requested transaction. This parameter specifies the
    ///   transaction for which the receipt is requested.
    ///
    /// ### Returns
    ///
    /// Returns a transaction receipt, which can be one of two types:
    /// - `TransactionReceipt` if the transaction has been processed and has a receipt.
    /// - `PendingTransactionReceipt` if the transaction is pending and the receipt is not yet
    ///   available.
    ///
    /// ### Errors
    ///
    /// The function may return a `TXN_HASH_NOT_FOUND` error if the specified transaction hash is
    /// not found.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: FieldElement,
    ) -> RpcResult<MaybePendingTransactionReceipt> {
        self.0.get_transaction_receipt(transaction_hash).await
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetWriteRpcApiServer for StarknetRpcWrapper<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    G: GenesisProvider + Send + Sync + 'static,
    H: HasherT + Send + Sync + 'static,
{
    /// Submit a new declare transaction to be added to the chain
    ///
    /// # Arguments
    ///
    /// * `declare_transaction` - the declare transaction to be added to the chain
    ///
    /// # Returns
    ///
    /// * `declare_transaction_result` - the result of the declare transaction
    async fn add_declare_transaction(
        &self,
        declare_transaction: BroadcastedDeclareTransaction,
    ) -> RpcResult<DeclareTransactionResult> {
        self.0.add_declare_transaction(declare_transaction).await
    }

    /// Add an Invoke Transaction to invoke a contract function
    ///
    /// # Arguments
    ///
    /// * `invoke tx` - <https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#invoke_transaction>
    ///
    /// # Returns
    ///
    /// * `transaction_hash` - transaction hash corresponding to the invocation
    async fn add_invoke_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTransaction,
    ) -> RpcResult<InvokeTransactionResult> {
        self.0.add_invoke_transaction(invoke_transaction).await
    }

    /// Add an Deploy Account Transaction
    ///
    /// # Arguments
    ///
    /// * `deploy account transaction` - <https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#deploy_account_transaction>
    ///
    /// # Returns
    ///
    /// * `transaction_hash` - transaction hash corresponding to the invocation
    /// * `contract_address` - address of the deployed contract account
    async fn add_deploy_account_transaction(
        &self,
        deploy_account_transaction: BroadcastedDeployAccountTransaction,
    ) -> RpcResult<DeployAccountTransactionResult> {
        self.0.add_deploy_account_transaction(deploy_account_transaction).await
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetTraceRpcApiServer for StarknetRpcWrapper<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    G: GenesisProvider + Send + Sync + 'static,
    H: HasherT + Send + Sync + 'static,
{
    /// Returns the execution trace of a transaction by simulating it in the runtime.
    async fn simulate_transactions(
        &self,
        block_id: BlockId,
        transactions: Vec<BroadcastedTransaction>,
        simulation_flags: Vec<SimulationFlag>,
    ) -> RpcResult<Vec<SimulatedTransaction>> {
        self.0.simulate_transactions(block_id, transactions, simulation_flags).await
    }

    /// Returns the execution traces of all transactions included in the given block
    async fn trace_block_transactions(&self, block_id: BlockId) -> RpcResult<Vec<TransactionTraceWithHash>> {
        self.0.trace_block_transactions(block_id).await
    }

    /// Returns the executions traces of a specified transaction in the given block
    async fn trace_transaction(&self, transaction_hash: FieldElement) -> RpcResult<TransactionTrace> {
        self.0.trace_transaction(transaction_hash).await
    }
}
