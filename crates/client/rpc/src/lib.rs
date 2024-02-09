//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod constants;
mod errors;
mod events;
mod madara_backend_client;
mod trace_api;
mod types;
use std::marker::PhantomData;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::types::error::CallError;
use log::error;
use mc_genesis_data_provider::GenesisProvider;
pub use mc_rpc_core::utils::*;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mc_storage::OverrideHandle;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::to_starknet_core_transaction::to_starknet_core_tx;
use mp_transactions::{TransactionStatus, UserTransaction};
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
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
use sp_runtime::DispatchError;
use starknet_api::block::BlockHash;
use starknet_api::hash::StarkHash;
use starknet_api::transaction::Calldata;
use starknet_core::types::{
    BlockHashAndNumber, BlockId, BlockStatus, BlockTag, BlockWithTxHashes, BlockWithTxs, BroadcastedDeclareTransaction,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction, ContractClass,
    DeclareTransactionReceipt, DeclareTransactionResult, DeployAccountTransactionReceipt,
    DeployAccountTransactionResult, EventFilterWithPage, EventsPage, ExecutionResources, ExecutionResult, FeeEstimate,
    FieldElement, FunctionCall, Hash256, InvokeTransactionReceipt, InvokeTransactionResult,
    L1HandlerTransactionReceipt, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs,
    MaybePendingTransactionReceipt, MsgFromL1, StateDiff, StateUpdate, SyncStatus, SyncStatusType, Transaction,
    TransactionExecutionStatus, TransactionFinalityStatus, TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;

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
        Ok("0.4.0".to_string())
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
        Ok(starknet_block.header().hash::<H>().into())
    }

    /// Returns the substrate block hash corresponding to the given Starknet block id
    fn substrate_block_hash_from_starknet_block(&self, block_id: BlockId) -> Result<B::Hash, StarknetRpcApiError> {
        match block_id {
            BlockId::Hash(h) => madara_backend_client::load_hash(self.client.as_ref(), &self.backend, h.into())
                .map_err(|e| {
                    error!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}");
                    StarknetRpcApiError::BlockNotFound
                })?,
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

    /// Returns a list of all transaction hashes in the given block.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The hash of the block containing the transactions (starknet block).
    fn get_cached_transaction_hashes(&self, block_hash: StarkHash) -> Option<Vec<StarkHash>> {
        self.backend.mapping().cached_transaction_hashes_from_block_hash(block_hash).unwrap_or_else(|err| {
            error!("Failed to read from cache: {err}");
            None
        })
    }

    /// Returns the state diff for the given block.
    ///
    /// # Arguments
    ///
    /// * `starknet_block_hash` - The hash of the block containing the state diff (starknet block).
    fn get_state_diff(&self, starknet_block_hash: &BlockHash) -> Result<StateDiff, StarknetRpcApiError> {
        let state_diff = self.backend.da().state_diff(starknet_block_hash).map_err(|e| {
            error!("Failed to retrieve state diff from cache for block with hash {}: {e}", starknet_block_hash);
            StarknetRpcApiError::InternalServerError
        })?;

        let rpc_state_diff = to_rpc_state_diff(state_diff);

        Ok(rpc_state_diff)
    }

    fn try_txn_hash_from_cache(
        &self,
        tx_index: usize,
        cached_transactions: &Option<Vec<StarkHash>>,
        transactions: &[mp_transactions::Transaction],
        chain_id: Felt252Wrapper,
    ) -> Result<Felt252Wrapper, StarknetRpcApiError> {
        if let Some(txn_hashes) = &cached_transactions {
            let txn_hash = (&txn_hashes
                .get(tx_index)
                .ok_or_else(|| {
                    error!("Failed to retrieve transaction hash from cache, invalid index {}", tx_index);
                    StarknetRpcApiError::InternalServerError
                })?
                .0)
                .try_into()
                .map_err(|_| {
                    error!("Failed to convert transaction hash");
                    StarknetRpcApiError::InternalServerError
                })?;
            Ok(txn_hash)
        } else {
            let transaction = &transactions.get(tx_index).ok_or_else(|| {
                error!("Failed to retrieve transaction hash from starknet txs, invalid index {}", tx_index);
                StarknetRpcApiError::InternalServerError
            })?;
            Ok(transaction.compute_hash::<H>(chain_id, false))
        }
    }
}

/// Taken from https://github.com/paritytech/substrate/blob/master/client/rpc/src/author/mod.rs#L78
const TX_SOURCE: TransactionSource = TransactionSource::External;

impl<A, B, BE, G, C, P, H> MadaraRpcApiServer for Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    G: GenesisProvider + Send + Sync + 'static,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    H: HasherT + Send + Sync + 'static,
{
    fn predeployed_accounts(&self) -> RpcResult<Vec<PredeployedAccountWithBalance>> {
        let genesis_data = self.genesis_provider.load_genesis_data()?;
        let block_id = BlockId::Tag(BlockTag::Latest);
        let fee_token_address: FieldElement = genesis_data.fee_token_address.0;

        Ok(genesis_data
            .predeployed_accounts
            .into_iter()
            .map(|account| {
                let contract_address: FieldElement = account.contract_address.into();
                let balance_string = &self
                    .call(
                        FunctionCall {
                            contract_address: fee_token_address,
                            entry_point_selector: get_selector_from_name("balanceOf")
                                .expect("the provided method name should be a valid ASCII string."),
                            calldata: vec![contract_address],
                        },
                        block_id,
                    )
                    .expect("FunctionCall attributes should be correct.")[0];
                let balance =
                    Felt252Wrapper::from_hex_be(balance_string).expect("`balanceOf` should return a Felt").into();
                PredeployedAccountWithBalance { account, balance }
            })
            .collect::<Vec<_>>())
    }
}

#[async_trait]
impl<A, B, BE, G, C, P, H> StarknetWriteRpcApiServer for Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
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
        let best_block_hash = self.client.info().best_hash;

        let opt_sierra_contract_class = if let BroadcastedDeclareTransaction::V2(ref tx) = declare_transaction {
            Some(flattened_sierra_to_sierra_contract_class(tx.contract_class.clone()))
        } else {
            None
        };

        let transaction: UserTransaction = declare_transaction.try_into().map_err(|e| {
            error!("Failed to convert BroadcastedDeclareTransaction to UserTransaction, error: {e}");
            StarknetRpcApiError::InternalServerError
        })?;
        let class_hash = match transaction {
            UserTransaction::Declare(ref tx, _) => tx.class_hash(),
            _ => Err(StarknetRpcApiError::InternalServerError)?,
        };

        let current_block_hash = self.client.info().best_hash;
        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), current_block_hash)
            .contract_class_by_class_hash(current_block_hash, (*class_hash).into());

        if let Some(contract_class) = contract_class {
            error!("Contract class already exists: {:?}", contract_class);
            return Err(StarknetRpcApiError::ClassAlreadyDeclared.into());
        }

        let extrinsic = convert_tx_to_extrinsic(self.client.clone(), best_block_hash, transaction.clone()).await?;

        submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await?;

        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let tx_hash = transaction.compute_hash::<H>(chain_id, false).into();

        if let Some(sierra_contract_class) = opt_sierra_contract_class {
            if let Some(e) = self
                .backend
                .sierra_classes()
                .store_sierra_class(Felt252Wrapper::from(class_hash.0).into(), sierra_contract_class)
                .err()
            {
                log::error!("Failed to store the sierra contract class for declare tx `{tx_hash:x}`: {e}")
            }
        }

        Ok(DeclareTransactionResult { transaction_hash: tx_hash, class_hash: class_hash.0 })
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
        let best_block_hash = self.client.info().best_hash;

        let transaction: UserTransaction = invoke_transaction.try_into().map_err(|e| {
            error!("Failed to convert BroadcastedInvokeTransaction to UserTransaction: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

        let extrinsic = convert_tx_to_extrinsic(self.client.clone(), best_block_hash, transaction.clone()).await?;

        submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await?;

        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        Ok(InvokeTransactionResult { transaction_hash: transaction.compute_hash::<H>(chain_id, false).into() })
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
        let best_block_hash = self.client.info().best_hash;

        let transaction: UserTransaction = deploy_account_transaction.try_into().map_err(|e| {
            error!("Failed to convert BroadcastedDeployAccountTransaction to UserTransaction, error: {e}",);
            StarknetRpcApiError::InternalServerError
        })?;

        let extrinsic = convert_tx_to_extrinsic(self.client.clone(), best_block_hash, transaction.clone()).await?;

        submit_extrinsic(self.pool.clone(), best_block_hash, extrinsic).await?;

        let chain_id = Felt252Wrapper(self.chain_id()?.0);
        let account_address = match &transaction {
            UserTransaction::DeployAccount(tx) => tx.account_address(),
            _ => Err(StarknetRpcApiError::InternalServerError)?,
        };

        Ok(DeployAccountTransactionResult {
            transaction_hash: transaction.compute_hash::<H>(chain_id, false).into(),
            contract_address: account_address.into(),
        })
    }
}

#[async_trait]
#[allow(unused_variables)]
impl<A, B, BE, G, C, P, H> StarknetReadRpcApiServer for Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
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
        let substrate_block_hash = self
            .backend
            .mapping()
            .block_hash_from_transaction_hash(Felt252Wrapper(transaction_hash).into())
            .map_err(|e| {
                error!("Failed to get transaction's substrate block hash from mapping_db: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?
            .ok_or(StarknetRpcApiError::TxnHashNotFound)?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let chain_id = self.chain_id()?.0.into();

        let starknet_tx =
            if let Some(tx_hashes) = self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into()) {
                tx_hashes
                    .into_iter()
                    .zip(starknet_block.transactions())
                    .find(|(tx_hash, _)| *tx_hash == Felt252Wrapper(transaction_hash).into())
                    .map(|(_, tx)| to_starknet_core_tx(tx.clone(), transaction_hash))
            } else {
                starknet_block
                    .transactions()
                    .iter()
                    .find(|tx| tx.compute_hash::<H>(chain_id, false).0 == transaction_hash)
                    .map(|tx| to_starknet_core_tx(tx.clone(), transaction_hash))
            };

        let execution_status = {
            let revert_error = self
                .client
                .runtime_api()
                .get_tx_execution_outcome(substrate_block_hash, Felt252Wrapper(transaction_hash).into())
                .map_err(|e| {
                    error!(
                        "Failed to get transaction execution outcome. Substrate block hash: {substrate_block_hash}, \
                         transaction hash: {transaction_hash}, error: {e}"
                    );
                    StarknetRpcApiError::InternalServerError
                })?;

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

        let runtime_api = self.client.runtime_api();

        let calldata = Calldata(Arc::new(request.calldata.iter().map(|x| Felt252Wrapper::from(*x).into()).collect()));

        let result = runtime_api
            .call(
                substrate_block_hash,
                Felt252Wrapper(request.contract_address).into(),
                Felt252Wrapper(request.entry_point_selector).into(),
                calldata,
            )
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let result = convert_error(self.client.clone(), substrate_block_hash, result)?;

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

        Ok(to_rpc_contract_class(contract_class).map_err(|e| {
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
                    let starting_block_hash = starting_block?.header().hash::<H>().0;

                    let current_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(best_number);
                    let current_block_hash = current_block?.header().hash::<H>().0;

                    let highest_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(highest_number);
                    let highest_block_hash = highest_block?.header().hash::<H>().0;

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

        Ok(to_rpc_contract_class(contract_class).map_err(|e| {
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let chain_id = self.chain_id()?;
        let starknet_version = starknet_block.header().protocol_version;
        let l1_gas_price = starknet_block.header().l1_gas_price;
        let block_hash = starknet_block.header().hash::<H>();

        let transaction_hashes = if let Some(tx_hashes) = self.get_cached_transaction_hashes(block_hash.into()) {
            let mut v = Vec::with_capacity(tx_hashes.len());
            for tx_hash in tx_hashes {
                v.push(FieldElement::from(tx_hash));
            }
            v
        } else {
            starknet_block.transactions_hashes::<H>(chain_id.0.into()).map(FieldElement::from).collect()
        };
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
            l1_gas_price: starknet_block.header().l1_gas_price.into(),
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
        let best_block_hash = self.client.info().best_hash;
        let chain_id = self.client.runtime_api().chain_id(best_block_hash).map_err(|e| {
            error!("Failed to fetch chain_id with best_block_hash: {best_block_hash}, error: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

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
        block_id: BlockId,
    ) -> RpcResult<Vec<FeeEstimate>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;
        let best_block_hash = self.client.info().best_hash;
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transactions =
            request.into_iter().map(|tx| tx.try_into()).collect::<Result<Vec<UserTransaction>, _>>().map_err(|e| {
                error!("Failed to convert BroadcastedTransaction to UserTransaction: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let fee_estimates = self
            .client
            .runtime_api()
            .estimate_fee(substrate_block_hash, transactions)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;

        let estimates = fee_estimates
            .into_iter()
			// FIXME: https://github.com/keep-starknet-strange/madara/issues/329
            .map(|x| FeeEstimate { gas_price: 10, gas_consumed: x.1, overall_fee: x.0 })
            .collect();

        Ok(estimates)
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

        let message = message.try_into().map_err(|e| {
            error!("Failed to convert MsgFromL1 to UserTransaction: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

        let fee_estimate = self
            .client
            .runtime_api()
            .estimate_message_fee(substrate_block_hash, message)
            .map_err(|e| {
                error!("Runtime api error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("function execution failed: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;

        let estimate = FeeEstimate {
            gas_price: fee_estimate.0.try_into().map_err(|_| StarknetRpcApiError::InternalServerError)?,
            gas_consumed: fee_estimate.2,
            overall_fee: fee_estimate.1,
        };

        Ok(estimate)
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
        let chain_id = self.chain_id()?;

        let opt_cached_transaction_hashes =
            self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into());

        let transaction_hash = if let Some(cached_tx_hashes) = opt_cached_transaction_hashes {
            cached_tx_hashes.get(index as usize).map(|&fe| FieldElement::from(fe)).ok_or(CallError::Failed(
                anyhow::anyhow!(
                    "Number of cached tx hashes does not match the number of transactions in block with id {:?}",
                    block_id
                ),
            ))?
        } else {
            transaction.compute_hash::<H>(chain_id.0.into(), false).0
        };

        Ok(to_starknet_core_tx(transaction.clone(), transaction_hash))
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
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("Block not found: '{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let block_hash = starknet_block.header().hash::<H>();
        let starknet_version = starknet_block.header().protocol_version;

        let chain_id = self.chain_id()?;
        let chain_id = Felt252Wrapper(chain_id.0);

        let opt_cached_transaction_hashes =
            self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into());
        let mut transactions = Vec::with_capacity(starknet_block.transactions().len());
        for (index, tx) in starknet_block.transactions().iter().enumerate() {
            let tx_hash = if let Some(cached_tx_hashes) = opt_cached_transaction_hashes.as_ref() {
                cached_tx_hashes.get(index).map(|&h| FieldElement::from(h)).ok_or(CallError::Failed(
                    anyhow::anyhow!(
                        "Number of cached tx hashes does not match the number of transactions in block with hash {:?}",
                        block_hash
                    ),
                ))?
            } else {
                tx.compute_hash::<H>(chain_id.0.into(), false).0
            };

            transactions.push(to_starknet_core_tx(tx.clone(), tx_hash));
        }

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
            l1_gas_price: starknet_block.header().l1_gas_price.into(),
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
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<StateUpdate> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let old_root = if starknet_block.header().block_number > 0 {
            let parent_block_hash = Felt252Wrapper::from(starknet_block.header().parent_block_hash).into();
            let substrate_parent_block_hash =
                self.substrate_block_hash_from_starknet_block(BlockId::Hash(parent_block_hash)).map_err(|e| {
                    error!("'{e}'");
                    StarknetRpcApiError::BlockNotFound
                })?;

            let parent_block = get_block_by_block_hash(self.client.as_ref(), substrate_parent_block_hash)?;

            Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into()
        } else {
            FieldElement::default()
        };

        let starknet_block_hash = BlockHash(starknet_block.header().hash::<H>().into());

        let state_diff = self.get_state_diff(&starknet_block_hash).map_err(|e| {
            error!("Failed to get state diff. Starknet block hash: {starknet_block_hash}, error: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

        Ok(StateUpdate {
            block_hash: starknet_block.header().hash::<H>().into(),
            new_root: Felt252Wrapper::from(self.backend.temporary_global_state_root_getter()).into(),
            old_root,
            state_diff,
        })
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
        let substrate_block_hash_from_db =
            self.backend.mapping().block_hash_from_transaction_hash(transaction_hash.into()).map_err(|e| {
                error!("Failed to get transaction's substrate block hash from mapping_db: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?;

        let substrate_block_hash = match substrate_block_hash_from_db {
            Some(block_hash) => block_hash,
            None => return Err(StarknetRpcApiError::TxnHashNotFound.into()),
        };

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let chain_id = self.chain_id()?.0.into();

        let find_tx =
            if let Some(tx_hashes) = self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into()) {
                tx_hashes
                    .into_iter()
                    .zip(starknet_block.transactions())
                    .find(|(tx_hash, _)| *tx_hash == Felt252Wrapper(transaction_hash).into())
                    .map(|(_, tx)| to_starknet_core_tx(tx.clone(), transaction_hash))
            } else {
                starknet_block
                    .transactions()
                    .iter()
                    .find(|tx| tx.compute_hash::<H>(chain_id, false).0 == transaction_hash)
                    .map(|tx| to_starknet_core_tx(tx.clone(), transaction_hash))
            };

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
        let substrate_block_hash = self
            .backend
            .mapping()
            .block_hash_from_transaction_hash(transaction_hash.into())
            .map_err(|e| {
                error!("Failed to interact with db backend error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .ok_or(StarknetRpcApiError::TxnHashNotFound)?;

        let starknet_block: mp_block::Block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let block_header = starknet_block.header();
        let block_hash = block_header.hash::<H>().into();
        let block_number = block_header.block_number;

        let block_extrinsics = self
            .client
            .block_body(substrate_block_hash)
            .map_err(|e| {
                error!("Failed to get block body. Substrate block hash: {substrate_block_hash}, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let chain_id = self.chain_id()?.0.into();

        let starknet_version = starknet_block.header().protocol_version;

        let fee_disabled =
            self.client.runtime_api().is_transaction_fee_disabled(substrate_block_hash).map_err(|e| {
                error!("Failed to get check fee disabled. Substrate block hash: {substrate_block_hash}, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let block_extrinsics_len = block_extrinsics.len();
        let transactions =
            self.client.runtime_api().extrinsic_filter(substrate_block_hash, block_extrinsics).map_err(|e| {
                error!("Failed to filter extrinsics. Substrate block hash: {substrate_block_hash}, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?;
        let txn_hashes = self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into());
        let mut tx_index = None;
        let mut transaction = None;
        for (index, tx) in transactions.iter().enumerate() {
            let tx_hash = self.try_txn_hash_from_cache(index, &txn_hashes, &transactions, chain_id)?;
            if tx_hash == transaction_hash.into() {
                tx_index = Some(index);
                transaction = Some(tx);
                break;
            }
        }
        if tx_index.is_none() || transaction.is_none() {
            error!(
                "Failed to find transaction hash in block. Substrate block hash: {substrate_block_hash}, transaction \
                 hash: {transaction_hash}"
            );
            return Err(StarknetRpcApiError::InternalServerError.into());
        }
        let tx_index = tx_index.unwrap();
        let transaction = transaction.unwrap();
        // adding the inherents count to the tx_index to get the correct index in the block
        let tx_index = tx_index + block_extrinsics_len - transactions.len();

        let events = self
            .client
            .runtime_api()
            .get_events_for_tx_by_index(substrate_block_hash, tx_index as u32)
            .map_err(|e| {
                error!(
                    "Failed to get events for transaction index. Substrate block hash: {substrate_block_hash}, \
                     transaction idx: {tx_index}, error: {e}"
                );
                StarknetRpcApiError::InternalServerError
            })?
            .expect("the transaction should be present in the substrate extrinsics"); // not reachable

        let execution_result = {
            let revert_error = self
                .client
                .runtime_api()
                .get_tx_execution_outcome(substrate_block_hash, Felt252Wrapper(transaction_hash).into())
                .map_err(|e| {
                    error!(
                        "Failed to get transaction execution outcome. Substrate block hash: {substrate_block_hash}, \
                         transaction hash: {transaction_hash}, error: {e}"
                    );
                    StarknetRpcApiError::InternalServerError
                })?;

            match revert_error {
                None => ExecutionResult::Succeeded,
                // This is safe because the message is a Vec<u8> build from a String
                Some(message) => ExecutionResult::Reverted { reason: unsafe { String::from_utf8_unchecked(message) } },
            }
        };

        // TODO(#1291): compute message hash correctly to L1HandlerTransactionReceipt
        let message_hash: Hash256 = Hash256::from_felt(&FieldElement::default());

        fn event_conversion(event: starknet_api::transaction::Event) -> starknet_core::types::Event {
            starknet_core::types::Event {
                from_address: Felt252Wrapper::from(event.from_address).0,
                keys: event.content.keys.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
                data: event.content.data.0.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
            }
        }

        let events_converted: Vec<starknet_core::types::Event> =
            events.clone().into_iter().map(event_conversion).collect();

        let actual_fee = if fee_disabled {
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
        };

        let messages = self
            .client
            .runtime_api()
            .get_tx_messages_to_l1(substrate_block_hash, Felt252Wrapper(transaction_hash).into())
            .map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::InternalServerError
            })?;

        fn message_conversion(message: starknet_api::transaction::MessageToL1) -> starknet_core::types::MsgToL1 {
            let mut to_address = [0u8; 32];
            to_address[12..32].copy_from_slice(message.to_address.0.as_bytes());
            starknet_core::types::MsgToL1 {
                from_address: Felt252Wrapper::from(message.from_address).0,
                to_address: FieldElement::from_bytes_be(&to_address).unwrap(),
                payload: message.payload.0.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
            }
        }

        // TODO: use actual execution ressources
        let receipt = match transaction {
            mp_transactions::Transaction::Declare(_) => TransactionReceipt::Declare(DeclareTransactionReceipt {
                transaction_hash,
                actual_fee,
                finality_status: TransactionFinalityStatus::AcceptedOnL2,
                block_hash,
                block_number,
                messages_sent: messages.into_iter().map(message_conversion).collect(),
                events: events_converted,
                execution_result,
                execution_resources: ExecutionResources {
                    steps: 0,
                    memory_holes: None,
                    range_check_builtin_applications: 0,
                    pedersen_builtin_applications: 0,
                    poseidon_builtin_applications: 0,
                    ec_op_builtin_applications: 0,
                    ecdsa_builtin_applications: 0,
                    bitwise_builtin_applications: 0,
                    keccak_builtin_applications: 0,
                },
            }),
            mp_transactions::Transaction::DeployAccount(tx) => {
                TransactionReceipt::DeployAccount(DeployAccountTransactionReceipt {
                    transaction_hash,
                    actual_fee,
                    finality_status: TransactionFinalityStatus::AcceptedOnL2,
                    block_hash,
                    block_number,
                    messages_sent: messages.into_iter().map(message_conversion).collect(),
                    events: events_converted,
                    contract_address: tx.get_account_address(),
                    execution_result,
                    execution_resources: ExecutionResources {
                        steps: 0,
                        memory_holes: None,
                        range_check_builtin_applications: 0,
                        pedersen_builtin_applications: 0,
                        poseidon_builtin_applications: 0,
                        ec_op_builtin_applications: 0,
                        ecdsa_builtin_applications: 0,
                        bitwise_builtin_applications: 0,
                        keccak_builtin_applications: 0,
                    },
                })
            }
            mp_transactions::Transaction::Invoke(_) => TransactionReceipt::Invoke(InvokeTransactionReceipt {
                transaction_hash,
                actual_fee,
                finality_status: TransactionFinalityStatus::AcceptedOnL2,
                block_hash,
                block_number,
                messages_sent: messages.into_iter().map(message_conversion).collect(),
                events: events_converted,
                execution_result,
                execution_resources: ExecutionResources {
                    steps: 0,
                    memory_holes: None,
                    range_check_builtin_applications: 0,
                    pedersen_builtin_applications: 0,
                    poseidon_builtin_applications: 0,
                    ec_op_builtin_applications: 0,
                    ecdsa_builtin_applications: 0,
                    bitwise_builtin_applications: 0,
                    keccak_builtin_applications: 0,
                },
            }),
            mp_transactions::Transaction::L1Handler(_) => TransactionReceipt::L1Handler(L1HandlerTransactionReceipt {
                message_hash,
                transaction_hash,
                actual_fee,
                finality_status: TransactionFinalityStatus::AcceptedOnL2,
                block_hash,
                block_number,
                messages_sent: messages.into_iter().map(message_conversion).collect(),
                events: events_converted,
                execution_result,
                execution_resources: ExecutionResources {
                    steps: 0,
                    memory_holes: None,
                    range_check_builtin_applications: 0,
                    pedersen_builtin_applications: 0,
                    poseidon_builtin_applications: 0,
                    ec_op_builtin_applications: 0,
                    ecdsa_builtin_applications: 0,
                    bitwise_builtin_applications: 0,
                    keccak_builtin_applications: 0,
                },
            }),
        };

        Ok(MaybePendingTransactionReceipt::Receipt(receipt))
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

async fn convert_tx_to_extrinsic<C, B>(
    client: Arc<C>,
    best_block_hash: <B as BlockT>::Hash,
    transaction: UserTransaction,
) -> Result<<B as BlockT>::Extrinsic, StarknetRpcApiError>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
{
    let extrinsic = client.runtime_api().convert_transaction(best_block_hash, transaction).map_err(|e| {
        error!("Failed to convert transaction: {:?}", e);
        StarknetRpcApiError::InternalServerError
    })?;

    Ok(extrinsic)
}

fn convert_error<C, B, T>(
    client: Arc<C>,
    best_block_hash: <B as BlockT>::Hash,
    call_result: Result<T, DispatchError>,
) -> Result<T, StarknetRpcApiError>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
{
    match call_result {
        Ok(val) => Ok(val),
        Err(e) => match client.runtime_api().convert_error(best_block_hash, e) {
            Ok(starknet_error) => Err(starknet_error.into()),
            Err(_) => Err(StarknetRpcApiError::InternalServerError),
        },
    }
}
