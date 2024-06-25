//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod constants;
mod errors;
mod events;
mod madara_backend_client;
mod madara_routes;
mod runtime_api;
pub mod starknetrpcwrapper;
mod trace_api;
mod types;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::objects::{ResourcesMapping, TransactionExecutionInfo};
use blockifier::transaction::transactions::{DeclareTransaction, L1HandlerTransaction};
use errors::StarknetRpcApiError;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_genesis_data_provider::GenesisProvider;
pub use mc_rpc_core::utils::*;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mc_storage::OverrideHandle;
use mp_block::BlockTransactions;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::SimulationFlags;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::from_broadcasted_transactions::{
    try_account_tx_from_broadcasted_tx, try_declare_tx_from_broadcasted_declare_tx,
    try_deploy_tx_from_broadcasted_deploy_tx, try_invoke_tx_from_broadcasted_invoke_tx,
};
use mp_transactions::to_starknet_core_transaction::to_starknet_core_tx;
use mp_transactions::{compute_message_hash, get_transaction_hash, TransactionStatus};
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_block_builder::GetPendingBlockExtrinsics;
use sc_client_api::backend::{Backend, StorageProvider};
use sc_client_api::BlockBackend;
use sc_network_sync::SyncingService;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::error::{Error as PoolError, IntoPoolError};
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::ProvideRuntimeApi;
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::transaction_validity::InvalidTransaction;
use starknet_api::core::{ClassHash, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, TransactionHash, TransactionVersion};
use starknet_core::types::{
    BlockHashAndNumber, BlockId, BlockStatus, BlockTag, BlockWithTxHashes, BlockWithTxs, BroadcastedDeclareTransaction,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction, ContractClass,
    DeclareTransactionReceipt, DeclareTransactionResult, DeployAccountTransactionReceipt,
    DeployAccountTransactionResult, EventFilterWithPage, EventsPage, ExecutionResources, ExecutionResult, FeeEstimate,
    FeePayment, FieldElement, FunctionCall, Hash256, InvokeTransactionReceipt, InvokeTransactionResult,
    L1HandlerTransactionReceipt, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
    MaybePendingTransactionReceipt, MsgFromL1, PendingBlockWithTxHashes, PendingBlockWithTxs,
    PendingDeclareTransactionReceipt, PendingDeployAccountTransactionReceipt, PendingInvokeTransactionReceipt,
    PendingL1HandlerTransactionReceipt, PendingStateUpdate, PendingTransactionReceipt, PriceUnit, ResourcePrice,
    SimulationFlagForEstimateFee, StateDiff, StateUpdate, SyncStatus, SyncStatusType, Transaction,
    TransactionExecutionStatus, TransactionFinalityStatus, TransactionReceipt,
};
use trace_api::get_previous_block_substrate_hash;

use crate::constants::{MAX_EVENTS_CHUNK_SIZE, MAX_EVENTS_KEYS};
use crate::types::RpcEventFilter;

/// A Starknet RPC server for Madara
pub struct Starknet<A: ChainApi, B: BlockT, BE, G, C, P, H> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
    pool: Arc<P>,
    #[allow(dead_code)]
    graph: Arc<Pool<A>>,
    sync_service: Arc<SyncingService<B>>,
    starting_block: <<B>::Header as HeaderT>::Number,
    genesis_provider: Arc<G>,
    _marker: PhantomData<(B, BE, H)>,
}

/// Constructor for A Starknet RPC server for Madara
/// # Arguments
// * `client` - The Madara client
// * `backend` - The Madara backend
// * `overrides` - The OverrideHandle
// * `sync_service` - The Substrate client sync service
// * `starting_block` - The starting block for the syncing
// * `hasher` - The hasher used by the runtime
//
// # Returns
// * `Self` - The actual Starknet struct
#[allow(clippy::too_many_arguments)]
impl<A: ChainApi, B: BlockT, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H> {
    pub fn new(
        client: Arc<C>,
        backend: Arc<mc_db::Backend<B>>,
        overrides: Arc<OverrideHandle<B>>,
        pool: Arc<P>,
        graph: Arc<Pool<A>>,
        sync_service: Arc<SyncingService<B>>,
        starting_block: <<B>::Header as HeaderT>::Number,
        genesis_provider: Arc<G>,
    ) -> Self {
        Self {
            client,
            backend,
            overrides,
            pool,
            graph,
            sync_service,
            starting_block,
            genesis_provider,
            _marker: PhantomData,
        }
    }
}

impl<A: ChainApi, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<A: ChainApi, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_spec_version(&self) -> RpcResult<String> {
        Ok("0.7.0".to_string())
    }
}

impl<A: ChainApi, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    H: HasherT + Send + Sync + 'static,
{
    pub fn current_block_hash(&self) -> Result<H256, StarknetRpcApiError> {
        let substrate_block_hash = self.client.info().best_hash;

        let starknet_block = match get_block_by_block_hash(self.client.as_ref(), substrate_block_hash) {
            Ok(block) => block,
            Err(_) => return Err(StarknetRpcApiError::BlockNotFound),
        };
        Ok(starknet_block.header().hash().into())
    }

    /// Returns the substrate block hash corresponding to the given Starknet block id
    fn substrate_block_hash_from_starknet_block(&self, block_id: BlockId) -> Result<B::Hash, StarknetRpcApiError> {
        match block_id {
            BlockId::Hash(h) => {
                madara_backend_client::load_hash(self.client.as_ref(), &self.backend, Felt252Wrapper(h).into())
                    .map_err(|e| {
                        error!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}");
                        StarknetRpcApiError::BlockNotFound
                    })?
            }
            BlockId::Number(n) => self
                .client
                .hash(UniqueSaturatedInto::unique_saturated_into(n))
                .map_err(|_| StarknetRpcApiError::BlockNotFound)?,
            BlockId::Tag(_) => Some(self.client.info().best_hash),
        }
        .ok_or(StarknetRpcApiError::BlockNotFound)
    }

    /// Helper function to get the substrate block number from a Starknet block id
    ///
    /// # Arguments
    ///
    /// * `block_id` - The Starknet block id
    ///
    /// # Returns
    ///
    /// * `u64` - The substrate block number
    fn substrate_block_number_from_starknet_block(&self, block_id: BlockId) -> Result<u64, StarknetRpcApiError> {
        // Short circuit on block number
        if let BlockId::Number(x) = block_id {
            return Ok(x);
        }

        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id)?;

        let starknet_block = match get_block_by_block_hash(self.client.as_ref(), substrate_block_hash) {
            Ok(block) => block,
            Err(_) => return Err(StarknetRpcApiError::BlockNotFound),
        };

        Ok(starknet_block.header().block_number)
    }

    fn get_current_resource_price(&self) -> Result<ResourcePrice, StarknetRpcApiError> {
        let current_prices =
            self.client.runtime_api().current_l1_gas_prices(self.client.info().best_hash).map_err(|e| {
                log::error!("Failed to get current L1 gas prices: {e}");
                StarknetRpcApiError::InternalServerError
            })?;
        Ok(ResourcePrice {
            price_in_wei: current_prices.eth_l1_gas_price.get().into(),
            price_in_fri: current_prices.strk_l1_gas_price.get().into(),
        })
    }
}

impl<A, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
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
    async fn declare_tx_common(
        &self,
        txn: DeclareTransaction,
    ) -> Result<(TransactionHash, ClassHash), StarknetRpcApiError> {
        let best_block_hash = self.get_best_block_hash();
        let current_block_hash = self.get_best_block_hash();
        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), current_block_hash)
            .contract_class_by_class_hash(current_block_hash, txn.class_hash());

        if let Some(contract_class) = contract_class {
            log::debug!("Contract class already exists: {:?}", contract_class);
            return Err(StarknetRpcApiError::ClassAlreadyDeclared);
        }

        let extrinsic =
            self.convert_tx_to_extrinsic(best_block_hash, AccountTransaction::Declare(txn.clone())).unwrap();

        let res = submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await;

        match res {
            Ok(_val) => Ok((txn.tx_hash, txn.class_hash())),
            Err(e) => Err(e),
        }
    }
}

/// Taken from https://github.com/paritytech/substrate/blob/master/client/rpc/src/author/mod.rs#L78
const TX_SOURCE: TransactionSource = TransactionSource::External;

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetWriteRpcApiServer for Starknet<A, B, BE, G, C, P, H>
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
        let opt_sierra_contract_class = if let BroadcastedDeclareTransaction::V2(ref tx) = declare_transaction {
            Some(flattened_sierra_to_sierra_contract_class(tx.contract_class.clone()))
        } else {
            None
        };

        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transaction = try_declare_tx_from_broadcasted_declare_tx(declare_transaction, chain_id).map_err(|e| {
            error!("Failed to convert BroadcastedDeclareTransaction to DeclareTransaction, error: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

        let (tx_hash, class_hash) = self.declare_tx_common(transaction).await?;

        if let Some(sierra_contract_class) = opt_sierra_contract_class {
            if let Some(e) = self.backend.sierra_classes().store_sierra_class(class_hash, sierra_contract_class).err() {
                log::error!("Failed to store the sierra contract class for declare tx `{tx_hash}`: {e}")
            }
        }

        Ok(DeclareTransactionResult {
            transaction_hash: Felt252Wrapper::from(tx_hash).into(),
            class_hash: Felt252Wrapper::from(class_hash).into(),
        })
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
        let best_block_hash = self.get_best_block_hash();
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transaction = try_invoke_tx_from_broadcasted_invoke_tx(invoke_transaction, chain_id).map_err(|e| {
            error!("Failed to convert BroadcastedInvokeTransaction to InvokeTransaction: {e}");
            StarknetRpcApiError::InternalServerError
        })?;
        let tx_hash = transaction.tx_hash;

        let extrinsic = self.convert_tx_to_extrinsic(best_block_hash, AccountTransaction::Invoke(transaction))?;

        submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await?;

        Ok(InvokeTransactionResult { transaction_hash: Felt252Wrapper::from(*tx_hash).into() })
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
        let best_block_hash = self.get_best_block_hash();
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transaction =
            try_deploy_tx_from_broadcasted_deploy_tx(deploy_account_transaction, chain_id).map_err(|e| {
                error!("Failed to convert BroadcastedDeployAccountTransaction to DeployAccountTransaction, error: {e}",);
                StarknetRpcApiError::InternalServerError
            })?;

        let (contract_address, tx_hash) = (transaction.contract_address, transaction.tx_hash);

        let extrinsic =
            self.convert_tx_to_extrinsic(best_block_hash, AccountTransaction::DeployAccount(transaction))?;

        submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await?;

        Ok(DeployAccountTransactionResult {
            transaction_hash: Felt252Wrapper::from(tx_hash).into(),
            contract_address: Felt252Wrapper::from(contract_address).into(),
        })
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetReadRpcApiServer for Starknet<A, B, BE, G, C, P, H>
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
        self.current_spec_version()
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
        self.current_block_number()
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
        let block_number = self.current_block_number()?;
        let block_hash = self.current_block_hash().map_err(|e| {
            error!("Failed to retrieve the current block hash: {}", e);
            StarknetRpcApiError::NoBlocks
        })?;

        Ok(BlockHashAndNumber {
            block_hash: FieldElement::from_byte_slice_be(block_hash.as_bytes()).unwrap(),
            block_number,
        })
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        Ok(starknet_block.header().transaction_count)
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
        let transaction_hash: TransactionHash = Felt252Wrapper(transaction_hash).into();

        let substrate_block_hash = self
            .backend
            .mapping()
            .block_hash_from_transaction_hash(transaction_hash)
            .map_err(|e| {
                error!("Failed to get transaction's substrate block hash from mapping_db: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?
            .ok_or(StarknetRpcApiError::TxnHashNotFound)?;

        let execution_status = {
            let revert_error = self.get_tx_execution_outcome(substrate_block_hash, transaction_hash)?;

            if revert_error.is_none() {
                TransactionExecutionStatus::Succeeded
            } else {
                TransactionExecutionStatus::Reverted
            }
        };

        Ok(TransactionStatus { finality_status: TransactionFinalityStatus::AcceptedOnL2, execution_status })
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address = Felt252Wrapper(contract_address).into();
        let key = Felt252Wrapper(key).into();

        let value = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .get_storage_by_storage_key(substrate_block_hash, contract_address, key)
            .ok_or_else(|| {
                error!("Failed to retrieve storage at '{contract_address:?}' and '{key:?}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(Felt(Felt252Wrapper::from(value).into()))
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let calldata = Calldata(Arc::new(request.calldata.iter().map(|x| Felt252Wrapper::from(*x).into()).collect()));
        let result = self.do_call(
            substrate_block_hash,
            Felt252Wrapper(request.contract_address).into(),
            Felt252Wrapper(request.entry_point_selector).into(),
            calldata,
        )?;

        Ok(result.iter().map(|x| format!("{:#x}", x.0)).collect())
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = Felt252Wrapper(contract_address).into();
        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_by_address(substrate_block_hash, contract_address_wrapped)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(blockifier_to_rpc_contract_class_types(contract_class).map_err(|e| {
            error!("Failed to convert contract class at '{contract_address}' to RPC contract class: {e}");
            StarknetRpcApiError::InvalidContractClass
        })?)
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address = Felt252Wrapper(contract_address).into();
        let class_hash = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_hash_by_address(substrate_block_hash, contract_address)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class hash at '{contract_address:?}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(Felt(Felt252Wrapper::from(class_hash).into()))
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
        // obtain best seen (highest) block number
        match self.sync_service.best_seen_block().await {
            Ok(best_seen_block) => {
                let best_number = self.client.info().best_number;
                let highest_number = best_seen_block.unwrap_or(best_number);

                // get a starknet block from the starting substrate block number
                let starting_block = madara_backend_client::starknet_block_from_substrate_hash(
                    self.client.as_ref(),
                    self.starting_block,
                );

                // get a starknet block from the current substrate block number
                let current_block =
                    madara_backend_client::starknet_block_from_substrate_hash(self.client.as_ref(), best_number);

                // get a starknet block from the highest substrate block number
                let highest_block =
                    madara_backend_client::starknet_block_from_substrate_hash(self.client.as_ref(), highest_number);

                if starting_block.is_ok() && current_block.is_ok() && highest_block.is_ok() {
                    // Convert block numbers and hashes to the respective type required by the `syncing` endpoint.
                    let starting_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(self.starting_block);
                    let starting_block_hash = starting_block?.header().hash().0;

                    let current_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(best_number);
                    let current_block_hash = current_block?.header().hash().0;

                    let highest_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(highest_number);
                    let highest_block_hash = highest_block?.header().hash().0;

                    // Build the `SyncStatus` struct with the respective syn information
                    Ok(SyncStatusType::Syncing(SyncStatus {
                        starting_block_num,
                        starting_block_hash,
                        current_block_num,
                        current_block_hash,
                        highest_block_num,
                        highest_block_hash,
                    }))
                } else {
                    // If there was an error when getting a starknet block, then we return `false`,
                    // as per the endpoint specification
                    log::error!("Failed to load Starknet block");
                    Ok(SyncStatusType::NotSyncing)
                }
            }
            Err(_) => {
                // If there was an error when getting a starknet block, then we return `false`,
                // as per the endpoint specification
                log::error!("`SyncingEngine` shut down");
                Ok(SyncStatusType::NotSyncing)
            }
        }
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let class_hash = Felt252Wrapper(class_hash).into();

        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_by_class_hash(substrate_block_hash, class_hash)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class from hash '{class_hash}'");
                StarknetRpcApiError::ClassHashNotFound
            })?;

        Ok(blockifier_to_rpc_contract_class_types(contract_class).map_err(|e| {
            error!("Failed to convert contract class from hash '{class_hash}' to RPC contract class: {e}");
            StarknetRpcApiError::InternalServerError
        })?)
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
        if is_pending_block(block_id) {
            let pending_block = self.prepare_pending_block_with_tx_hashes()?;
            return Ok(MaybePendingBlockWithTxHashes::PendingBlock(pending_block));
        }

        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;
        let starknet_version = starknet_block.header().protocol_version;
        let block_hash = starknet_block.header().hash();

        let transaction_hashes =
            starknet_block.transactions_hashes().map(|txh| Felt252Wrapper::from(*txh).into()).collect();
        let block_status = match self.backend.messaging().last_synced_l1_block_with_event() {
            Ok(l1_block) => {
                if l1_block.block_number >= starknet_block.header().block_number {
                    BlockStatus::AcceptedOnL1
                } else {
                    BlockStatus::AcceptedOnL2
                }
            }
            Err(e) => {
                error!("Failed to get last synced l1 block, error: {e}");
                Err(StarknetRpcApiError::InternalServerError)?
            }
        };

        let parent_blockhash = starknet_block.header().parent_block_hash;
        let block_with_tx_hashes = BlockWithTxHashes {
            transactions: transaction_hashes,
            status: block_status,
            block_hash: block_hash.into(),
            parent_hash: Felt252Wrapper::from(parent_blockhash).into(),
            block_number: starknet_block.header().block_number,
            new_root: Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into(),
            timestamp: starknet_block.header().block_timestamp,
            sequencer_address: Felt252Wrapper::from(starknet_block.header().sequencer_address).into(),
            l1_gas_price: self.get_current_resource_price()?,
            starknet_version: starknet_version.to_string(),
        };

        Ok(MaybePendingBlockWithTxHashes::Block(block_with_tx_hashes))
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address = Felt252Wrapper(contract_address).into();

        let nonce = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .nonce(substrate_block_hash, contract_address)
            .ok_or_else(|| {
                error!("Failed to get nonce at '{contract_address:?}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(Felt(Felt252Wrapper::from(nonce).into()))
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
        let best_block_hash = self.get_best_block_hash();
        let chain_id = self.get_chain_id(best_block_hash)?;

        Ok(Felt(chain_id.0))
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transactions = request
            .into_iter()
            .map(|tx| try_account_tx_from_broadcasted_tx(tx, chain_id))
            .collect::<Result<Vec<AccountTransaction>, _>>()
            .map_err(|e| {
                error!("Failed to convert BroadcastedTransaction to AccountTransaction: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let fee_estimates =
            self.estimate_fee(substrate_block_hash, transactions, SimulationFlags::from(simulation_flags))?;

        Ok(fee_estimates)
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transaction = {
            let calldata = std::iter::once(Felt252Wrapper::from(message.from_address).into())
                .chain(message.payload.into_iter().map(|felt| Felt252Wrapper::from(felt).into()))
                .collect();
            let tx = starknet_api::transaction::L1HandlerTransaction {
                version: TransactionVersion::ZERO,
                // Nonce is not used during the message fee estimation.
                // Just put whatever.
                nonce: Nonce(StarkFelt::ZERO),
                contract_address: Felt252Wrapper::from(message.to_address).into(),
                entry_point_selector: Felt252Wrapper::from(message.entry_point_selector).into(),
                calldata: Calldata(Arc::new(calldata)),
            };
            let tx_hash = tx.compute_hash(chain_id, true);

            // Hardcoded `paid_fee_on_l1` value as it is not relevant here
            L1HandlerTransaction { tx, tx_hash, paid_fee_on_l1: Fee(1) }
        };

        let fee_estimate = self.do_estimate_message_fee(substrate_block_hash, transaction)?;

        Ok(fee_estimate)
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let transaction =
            starknet_block.transactions().get(index as usize).ok_or(StarknetRpcApiError::InvalidTxnIndex)?;

        Ok(to_starknet_core_tx(transaction.clone()))
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
        if is_pending_block(block_id) {
            let pending_block = self.prepare_pending_block_with_txs()?;
            return Ok(MaybePendingBlockWithTxs::PendingBlock(pending_block));
        }

        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("Block not found: '{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let block_hash = starknet_block.header().hash();
        let starknet_version = starknet_block.header().protocol_version;
        let transactions = starknet_block.transactions().iter().map(|tx| to_starknet_core_tx(tx.clone())).collect();

        let block_with_txs = BlockWithTxs {
            // TODO: Get status from block
            status: BlockStatus::AcceptedOnL2,
            block_hash: block_hash.into(),
            parent_hash: Felt252Wrapper::from(starknet_block.header().parent_block_hash).into(),
            block_number: starknet_block.header().block_number,
            new_root: Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into(),
            timestamp: starknet_block.header().block_timestamp,
            sequencer_address: Felt252Wrapper::from(starknet_block.header().sequencer_address).into(),
            transactions,
            l1_gas_price: self.get_current_resource_price()?,
            starknet_version: starknet_version.to_string(),
        };

        Ok(MaybePendingBlockWithTxs::Block(block_with_txs))
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
        if is_pending_block(block_id) {
            let state_diff = StateDiff {
                storage_diffs: Vec::new(),
                deprecated_declared_classes: Vec::new(),
                declared_classes: Vec::new(),
                deployed_contracts: Vec::new(),
                replaced_classes: Vec::new(),
                nonces: Vec::new(),
            };

            let old_root = Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into();
            let pending_state_update = PendingStateUpdate { old_root, state_diff };

            return Ok(MaybePendingStateUpdate::PendingUpdate(pending_state_update));
        }

        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let old_root = if starknet_block.header().block_number > 0 {
            Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into()
        } else {
            FieldElement::default()
        };

        let block_transactions = starknet_block.transactions();

        let previous_block_substrate_hash = get_previous_block_substrate_hash(self, substrate_block_hash)?;

        let state_diff = self.get_transaction_re_execution_state_diff(
            previous_block_substrate_hash,
            vec![],
            block_transactions.clone(),
        )?;

        let state_update = StateUpdate {
            block_hash: starknet_block.header().hash().into(),
            new_root: Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into(),
            old_root,
            state_diff,
        };

        Ok(MaybePendingStateUpdate::Update(state_update))
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
        let continuation_token = match filter.result_page_request.continuation_token {
            Some(token) => types::ContinuationToken::parse(token).map_err(|e| {
                error!("Failed to parse continuation token: {:?}", e);
                StarknetRpcApiError::InvalidContinuationToken
            })?,
            None => types::ContinuationToken::default(),
        };
        let from_address = filter.event_filter.address.map(Felt252Wrapper::from);
        let keys = filter.event_filter.keys.unwrap_or_default();
        let chunk_size = filter.result_page_request.chunk_size;

        if keys.len() > MAX_EVENTS_KEYS {
            return Err(StarknetRpcApiError::TooManyKeysInFilter.into());
        }
        if chunk_size > MAX_EVENTS_CHUNK_SIZE as u64 {
            return Err(StarknetRpcApiError::PageSizeTooBig.into());
        }

        // Get the substrate block numbers for the requested range
        let latest_block =
            self.substrate_block_number_from_starknet_block(BlockId::Tag(BlockTag::Latest)).map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;
        let from_block = self
            .substrate_block_number_from_starknet_block(filter.event_filter.from_block.unwrap_or(BlockId::Number(0)))
            .map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;
        let to_block = self
            .substrate_block_number_from_starknet_block(
                filter.event_filter.to_block.unwrap_or(BlockId::Tag(BlockTag::Latest)),
            )
            .map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;

        // Verify that the requested range is valid
        if from_block > to_block {
            return Ok(EventsPage { events: vec![], continuation_token: None });
        }

        let to_block = if latest_block > to_block { to_block } else { latest_block };
        let filter = RpcEventFilter { from_block, to_block, from_address, keys, chunk_size, continuation_token };

        self.filter_events(filter)
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
        let substrate_block_hash_from_db = self
            .backend
            .mapping()
            .block_hash_from_transaction_hash(Felt252Wrapper::from(transaction_hash).into())
            .map_err(|e| {
                error!("Failed to get transaction's substrate block hash from mapping_db: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?;

        let substrate_block_hash = match substrate_block_hash_from_db {
            Some(block_hash) => block_hash,
            None => return Err(StarknetRpcApiError::TxnHashNotFound.into()),
        };

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let searched_tx_hash: TransactionHash = Felt252Wrapper::from(transaction_hash).into();
        let find_tx = starknet_block
            .transactions()
            .iter()
            .find(|tx| get_transaction_hash(tx) == &searched_tx_hash)
            .map(|tx| to_starknet_core_tx(tx.clone()));

        find_tx.ok_or(StarknetRpcApiError::TxnHashNotFound.into())
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
        let transaction_hash = Felt252Wrapper::from(transaction_hash).into();

        let receipt = match self.backend.mapping().block_hash_from_transaction_hash(transaction_hash).map_err(|e| {
            error!("Failed to interact with db backend error: {e}");
            StarknetRpcApiError::InternalServerError
        })? {
            Some(substrate_block_hash) => self.prepare_tx_receipt(transaction_hash, substrate_block_hash).await?,
            // Try to find pending Tx
            None => self.get_pending_transaction_receipt(transaction_hash).await.map_err(|e| {
                error!("Failed to find pending tx with hash: {transaction_hash}: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?,
        };
        Ok(receipt)
    }
}

/// RPC Helper methods
impl<A, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
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
    fn prepare_pending_block_with_tx_hashes(&self) -> Result<PendingBlockWithTxHashes, StarknetRpcApiError> {
        let parent_hash = self.get_best_block_hash();
        let latest_block = get_block_by_block_hash(self.client.as_ref(), parent_hash)
            .map_err(|_| StarknetRpcApiError::BlockNotFound)?;
        let latest_block_header = latest_block.header();
        let transaction_hashes = self
            .get_pending_txs(parent_hash)?
            .iter()
            .map(|tx| Felt252Wrapper::from(*get_transaction_hash(tx)).into())
            .collect::<Vec<_>>();

        let pending_block = PendingBlockWithTxHashes {
            transactions: transaction_hashes,
            l1_gas_price: self.get_current_resource_price()?,
            parent_hash: latest_block_header.hash().into(),
            sequencer_address: Felt252Wrapper::from(latest_block_header.sequencer_address).into(),
            starknet_version: latest_block_header.protocol_version.to_string(),
            timestamp: calculate_pending_block_timestamp(),
        };
        Ok(pending_block)
    }

    fn prepare_pending_block_with_txs(&self) -> Result<PendingBlockWithTxs, StarknetRpcApiError> {
        let parent_hash = self.get_best_block_hash();
        let latest_block = get_block_by_block_hash(self.client.as_ref(), parent_hash)
            .map_err(|_| StarknetRpcApiError::BlockNotFound)?;
        let latest_block_header = latest_block.header();

        let transactions =
            self.get_pending_txs(parent_hash)?.iter().map(|tx| to_starknet_core_tx(tx.clone())).collect::<Vec<_>>();

        let pending_block = PendingBlockWithTxs {
            transactions,
            l1_gas_price: self.get_current_resource_price()?,
            parent_hash: latest_block_header.hash().into(),
            sequencer_address: Felt252Wrapper::from(latest_block_header.sequencer_address).into(),
            starknet_version: latest_block_header.protocol_version.to_string(),
            timestamp: calculate_pending_block_timestamp(),
        };
        Ok(pending_block)
    }

    fn get_pending_txs(
        &self,
        latest_block: B::Hash,
    ) -> Result<Vec<blockifier::transaction::transaction_execution::Transaction>, StarknetRpcApiError> {
        let pending_transactions: Vec<B::Extrinsic> = self.client.get_pending_extrinsics();

        // Use Runtime API to filter all Pending Txs
        // And get only Starknet Txs (Pallet Starknet calls) as
        // Vec<blockifier::transaction::transaction_execution::Transaction>
        self.filter_extrinsics(latest_block, pending_transactions)
    }

    async fn prepare_tx_receipt(
        &self,
        transaction_hash: TransactionHash,
        substrate_block_hash: B::Hash,
    ) -> Result<MaybePendingTransactionReceipt, StarknetRpcApiError> {
        let starknet_block: mp_block::Block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)
            .map_err(|_e| StarknetRpcApiError::BlockNotFound)?;
        let block_header = starknet_block.header();
        let block_hash = block_header.hash().into();
        let block_number = block_header.block_number;

        let transaction =
            starknet_block.transactions().iter().find(|tx| get_transaction_hash(tx) == &transaction_hash).ok_or_else(
                || {
                    error!(
                        "Failed to find transaction hash in block. Substrate block hash: {substrate_block_hash}, \
                         transaction hash: {transaction_hash}"
                    );
                    StarknetRpcApiError::InternalServerError
                },
            )?;

        let events = self.get_events_for_tx_by_hash(substrate_block_hash, transaction_hash)?;

        let execution_result = {
            let revert_error = self.get_tx_execution_outcome(substrate_block_hash, transaction_hash)?;

            // This is safe because the message is a Vec<u8> build from a String
            revert_error_to_execution_result(
                revert_error.map(|message| unsafe { String::from_utf8_unchecked(message) }),
            )
        };

        let events_converted: Vec<starknet_core::types::Event> =
            events.clone().into_iter().map(starknet_api_to_starknet_core_event).collect();

        let fee_disabled = self.is_transaction_fee_disabled(substrate_block_hash)?;
        let actual_fee = FeePayment {
            amount: if fee_disabled {
                FieldElement::ZERO
            } else {
                // Event {
                //     from_address: fee_token_address,
                //     keys: [selector("Transfer")],
                //     data: [
                //         send_from_address,       // account_contract_address
                //         send_to_address,         // to (sequencer address)
                //         expected_fee_value_low,  // transfer amount (fee)
                //         expected_fee_value_high,
                //     ]},
                // fee transfer must be the last event, except enabled disable-transaction-fee feature
                events_converted.last().unwrap().data[2]
            },
            unit: PriceUnit::Wei,
        };

        let messages = self.get_tx_messages_to_l1(substrate_block_hash, transaction_hash)?;

        let messages_sent = messages.into_iter().map(starknet_api_to_starknet_core_message_to_l1).collect();

        let parent_block_hash = self
            .substrate_block_hash_from_starknet_block(BlockId::Hash(
                Felt252Wrapper::from(block_header.parent_block_hash).into(),
            ))
            .map_err(|e| {
                error!("Parent Block not found: {e}");
                StarknetRpcApiError::BlockNotFound
            })?;
        let execution_info =
            self.get_transaction_execution_info(parent_block_hash, starknet_block.transactions(), transaction_hash)?;
        let execution_resources = actual_resources_to_execution_resources(execution_info.actual_resources);
        let transaction_hash = Felt252Wrapper::from(transaction_hash).into();

        let receipt = match transaction {
            blockifier::transaction::transaction_execution::Transaction::AccountTransaction(account_tx) => {
                match account_tx {
                    blockifier::transaction::account_transaction::AccountTransaction::Declare(_) => {
                        TransactionReceipt::Declare(DeclareTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            finality_status: TransactionFinalityStatus::AcceptedOnL2,
                            block_hash,
                            block_number,
                            messages_sent,
                            events: events_converted,
                            execution_result,
                            execution_resources,
                        })
                    }
                    blockifier::transaction::account_transaction::AccountTransaction::DeployAccount(tx) => {
                        TransactionReceipt::DeployAccount(DeployAccountTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            finality_status: TransactionFinalityStatus::AcceptedOnL2,
                            block_hash,
                            block_number,
                            messages_sent,
                            events: events_converted,
                            contract_address: Felt252Wrapper::from(tx.contract_address).into(),
                            execution_result,
                            execution_resources,
                        })
                    }
                    blockifier::transaction::account_transaction::AccountTransaction::Invoke(_) => {
                        TransactionReceipt::Invoke(InvokeTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            finality_status: TransactionFinalityStatus::AcceptedOnL2,
                            block_hash,
                            block_number,
                            messages_sent,
                            events: events_converted,
                            execution_result,
                            execution_resources,
                        })
                    }
                }
            }
            blockifier::transaction::transaction_execution::Transaction::L1HandlerTransaction(l1_handler_tx) => {
                let message_hash = compute_message_hash(&l1_handler_tx.tx);
                TransactionReceipt::L1Handler(L1HandlerTransactionReceipt {
                    message_hash: Hash256::from_bytes(message_hash.to_fixed_bytes()),
                    transaction_hash,
                    actual_fee,
                    finality_status: TransactionFinalityStatus::AcceptedOnL2,
                    block_hash,
                    block_number,
                    messages_sent,
                    events: events_converted,
                    execution_result,
                    execution_resources,
                })
            }
        };

        Ok(MaybePendingTransactionReceipt::Receipt(receipt))
    }

    fn get_transaction_execution_info(
        &self,
        parent_substrate_block_hash: B::Hash,
        block_transactions: &BlockTransactions,
        transaction_hash: TransactionHash,
    ) -> Result<TransactionExecutionInfo, StarknetRpcApiError>
    where
        B: BlockT,
    {
        let (transactions_before, transaction_to_trace) =
            split_block_tx_for_reexecution(block_transactions, transaction_hash).map_err(|e| {
                log::error!("Failed to split block transactions for re-execution: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        if transaction_to_trace.is_empty() {
            return Err(StarknetRpcApiError::TxnHashNotFound);
        }

        if transaction_to_trace.len() > 1 {
            log::error!("More than one transaction with the same transaction hash {:#?}", transaction_to_trace);
            return Err(StarknetRpcApiError::InternalServerError);
        }

        let mut trace = self
            .re_execute_transactions(parent_substrate_block_hash, transactions_before, transaction_to_trace, false)
            .map_err(|e| {
                log::error!("Failed to re-execute transactions: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let execution_info = trace.remove(0);

        Ok(execution_info.0)
    }

    fn get_events_for_tx_by_hash(
        &self,
        substrate_block_hash: B::Hash,
        tx_hash: TransactionHash,
    ) -> Result<Vec<starknet_api::transaction::Event>, StarknetRpcApiError> {
        let events = self.do_get_events_for_tx_by_hash(substrate_block_hash, tx_hash)?;
        Ok(events)
    }

    fn get_tx_execution_outcome(
        &self,
        substrate_block_hash: B::Hash,
        transaction_hash: TransactionHash,
    ) -> Result<Option<Vec<u8>>, StarknetRpcApiError> {
        self.do_get_tx_execution_outcome(substrate_block_hash, transaction_hash)
    }

    fn find_pending_tx(
        &self,
        tx_hash: TransactionHash,
    ) -> Result<Option<blockifier::transaction::transaction_execution::Transaction>, StarknetRpcApiError> {
        let latest_block = self.get_best_block_hash();

        let pending_tx =
            self.get_pending_txs(latest_block)?.iter().find(|&tx| get_transaction_hash(tx) == &tx_hash).cloned();

        Ok(pending_tx)
    }

    async fn get_pending_transaction_receipt(
        &self,
        transaction_hash: TransactionHash,
    ) -> Result<MaybePendingTransactionReceipt, StarknetRpcApiError> {
        let pending_tx = self.find_pending_tx(transaction_hash)?.ok_or(StarknetRpcApiError::TxnHashNotFound)?;

        // TODO: Massa labs is working on pending blocks within Substrate. That will allow fetching
        // events and messages directly from the runtime the same way we do for finalized blocks.
        // So for now we return empty events and messages. Another option is to expose the event and message
        // ordering functions from the runtime, order events inside execution info and use it. But the
        // effort will not be worth it after pending blocks, so we've skipped implementing this for
        // now.
        let messages_sent = Vec::new();
        let events = Vec::new();

        let parent_substrate_block_hash = self.get_best_block_hash();
        let pending_txs = self.get_pending_txs(parent_substrate_block_hash)?;
        let simulation =
            self.get_transaction_execution_info(parent_substrate_block_hash, &pending_txs, transaction_hash)?;
        let actual_fee =
            FeePayment { amount: Felt252Wrapper::from(simulation.actual_fee.0).into(), unit: PriceUnit::Wei };
        let execution_result = revert_error_to_execution_result(simulation.revert_error);
        let execution_resources = actual_resources_to_execution_resources(simulation.actual_resources);
        let transaction_hash = Felt252Wrapper::from(transaction_hash).into();

        let receipt = match pending_tx {
            blockifier::transaction::transaction_execution::Transaction::AccountTransaction(account_tx) => {
                match account_tx {
                    AccountTransaction::Declare(_tx) => {
                        let receipt = PendingDeclareTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            messages_sent,
                            events,
                            execution_resources,
                            execution_result,
                        };
                        PendingTransactionReceipt::Declare(receipt)
                    }
                    AccountTransaction::DeployAccount(tx) => {
                        let contract_address = Felt252Wrapper::from(tx.contract_address).into();
                        let receipt = PendingDeployAccountTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            messages_sent,
                            events,
                            execution_resources,
                            execution_result,
                            contract_address,
                        };
                        PendingTransactionReceipt::DeployAccount(receipt)
                    }
                    AccountTransaction::Invoke(_tx) => {
                        let receipt = PendingInvokeTransactionReceipt {
                            transaction_hash,
                            actual_fee,
                            messages_sent,
                            events,
                            execution_resources,
                            execution_result,
                        };
                        PendingTransactionReceipt::Invoke(receipt)
                    }
                }
            }
            blockifier::transaction::transaction_execution::Transaction::L1HandlerTransaction(tx) => {
                let message_hash = Hash256::from_bytes(compute_message_hash(&tx.tx).to_fixed_bytes());
                let receipt = PendingL1HandlerTransactionReceipt {
                    message_hash,
                    transaction_hash,
                    actual_fee,
                    messages_sent,
                    events,
                    execution_resources,
                    execution_result,
                };
                PendingTransactionReceipt::L1Handler(receipt)
            }
        };

        Ok(MaybePendingTransactionReceipt::PendingReceipt(receipt))
    }
}

async fn submit_extrinsic<P, B>(
    pool: Arc<P>,
    best_block_hash: <B as BlockT>::Hash,
    extrinsic: <B as BlockT>::Extrinsic,
) -> Result<<P as TransactionPool>::Hash, StarknetRpcApiError>
where
    P: TransactionPool<Block = B> + 'static,
    B: BlockT,
    <B as BlockT>::Extrinsic: Send + Sync + 'static,
{
    pool.submit_one(best_block_hash, TX_SOURCE, extrinsic).await.map_err(|e| {
        error!("Failed to submit extrinsic: {:?}", e);
        match e.into_pool_error() {
            Ok(PoolError::InvalidTransaction(InvalidTransaction::BadProof)) => StarknetRpcApiError::ValidationFailure,
            _ => StarknetRpcApiError::InternalServerError,
        }
    })
}

/// The current timestamp in seconds.
fn calculate_pending_block_timestamp() -> u64 {
    let timestamp_in_millisecond = sp_timestamp::InherentDataProvider::from_system_time().as_millis();
    timestamp_in_millisecond / 1000
}

fn is_pending_block(block_id: BlockId) -> bool {
    block_id == BlockId::Tag(BlockTag::Pending)
}

fn starknet_api_to_starknet_core_event(event: starknet_api::transaction::Event) -> starknet_core::types::Event {
    starknet_core::types::Event {
        from_address: Felt252Wrapper::from(event.from_address).0,
        keys: event.content.keys.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
        data: event.content.data.0.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
    }
}

fn starknet_api_to_starknet_core_message_to_l1(
    message: starknet_api::transaction::MessageToL1,
) -> starknet_core::types::MsgToL1 {
    let mut to_address = [0u8; 32];
    to_address[12..32].copy_from_slice(message.to_address.0.as_bytes());
    starknet_core::types::MsgToL1 {
        from_address: Felt252Wrapper::from(message.from_address).0,
        to_address: FieldElement::from_bytes_be(&to_address).unwrap(),
        payload: message.payload.0.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
    }
}

fn revert_error_to_execution_result(revert_error: Option<String>) -> ExecutionResult {
    match revert_error {
        None => ExecutionResult::Succeeded,
        Some(message) => ExecutionResult::Reverted { reason: message },
    }
}

fn actual_resources_to_execution_resources(resources: ResourcesMapping) -> ExecutionResources {
    let resources =
        resources.0.into_iter().map(|(k, v)| (k.to_lowercase(), v as u64)).collect::<HashMap<String, u64>>();
    // Based on `VM_RESOURCE_FEE_COSTS`
    // in crates/primitives/fee/src/lib.rs
    ExecutionResources {
        steps: resources.get("n_steps").cloned().unwrap_or_default(),
        memory_holes: resources.get("memory_holes").copied(),
        range_check_builtin_applications: resources.get("range_check_builtin").cloned(),
        pedersen_builtin_applications: resources.get("pedersen_builtin").cloned(),
        poseidon_builtin_applications: resources.get("poseidon_builtin").cloned(),
        ec_op_builtin_applications: resources.get("ec_op_builtin").cloned(),
        ecdsa_builtin_applications: resources.get("ecdsa_builtin").cloned(),
        bitwise_builtin_applications: resources.get("bitwise_builtin").cloned(),
        keccak_builtin_applications: resources.get("keccak_builtin").cloned(),
        segment_arena_builtin: resources.get("segment_arena_builtin").cloned(),
    }
}

fn split_block_tx_for_reexecution(
    block_transactions: &BlockTransactions,
    transaction_hash: TransactionHash,
) -> RpcResult<(
    Vec<blockifier::transaction::transaction_execution::Transaction>,
    Vec<blockifier::transaction::transaction_execution::Transaction>,
)> {
    let tx_to_trace_idx = block_transactions
        .iter()
        .rposition(|tx| get_transaction_hash(tx) == &transaction_hash)
        .ok_or(StarknetRpcApiError::TxnHashNotFound)?;

    Ok((block_transactions[0..tx_to_trace_idx].to_vec(), vec![block_transactions[tx_to_trace_idx].clone()]))
}
