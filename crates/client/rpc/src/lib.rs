//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_rpc_core::utils::{to_declare_tx, to_deploy_account_tx, to_invoke_tx, to_rpc_contract_class, to_tx};
pub use mc_rpc_core::StarknetRpcApiServer;
use mc_storage::OverrideHandle;
use mp_starknet::block::BlockTransactions;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::traits::hash::HasherT;
use mp_starknet::traits::ThreadSafeCopy;
use mp_starknet::transaction::types::{RPCTransactionConversionError, Transaction as MPTransaction, TxType};
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::{Backend, StorageProvider};
use sc_network_sync::SyncingService;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::generic::BlockId as SPBlockId;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use starknet_core::types::{
    BlockHashAndNumber, BlockId, BlockStatus, BlockTag, BlockWithTxHashes, BlockWithTxs, BroadcastedDeclareTransaction,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction, ContractClass,
    DeclareTransactionResult, DeployAccountTransactionResult, EventFilter, EventsPage, FeeEstimate, FieldElement,
    FunctionCall, InvokeTransactionResult, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, StateUpdate,
    SyncStatus, SyncStatusType, Transaction,
};

/// A Starknet RPC server for Madara
pub struct Starknet<B: BlockT, BE, C, P, H> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
    pool: Arc<P>,
    sync_service: Arc<SyncingService<B>>,
    starting_block: <<B>::Header as HeaderT>::Number,
    hasher: Arc<H>,
    _marker: PhantomData<(B, BE)>,
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
impl<B: BlockT, BE, C, P, H> Starknet<B, BE, C, P, H> {
    pub fn new(
        client: Arc<C>,
        backend: Arc<mc_db::Backend<B>>,
        overrides: Arc<OverrideHandle<B>>,
        pool: Arc<P>,
        sync_service: Arc<SyncingService<B>>,
        starting_block: <<B>::Header as HeaderT>::Number,
        hasher: Arc<H>,
    ) -> Self {
        Self { client, backend, overrides, pool, sync_service, starting_block, hasher, _marker: PhantomData }
    }
}

impl<B, BE, C, P, H> Starknet<B, BE, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, BE, C, P, H> Starknet<B, BE, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    BE: Backend<B>,
    H: HasherT + ThreadSafeCopy,
{
    pub fn current_block_hash(&self) -> Result<H256, ApiError> {
        let substrate_block_hash = self.client.info().best_hash;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        Ok(block.header().hash(*self.hasher).into())
    }

    /// Returns the substrate block corresponding to the given Starknet block id
    fn substrate_block_hash_from_starknet_block(&self, block_id: BlockId) -> Result<B::Hash, String> {
        match block_id {
            BlockId::Hash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_slice(&h.to_bytes_be()[..32]),
            )
            .map_err(|e| format!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}"))?,
            BlockId::Number(n) => self
                .client
                .hash(UniqueSaturatedInto::unique_saturated_into(n))
                .map_err(|e| format!("Failed to retrieve the hash of block number '{n}': {e}"))?,
            BlockId::Tag(t) => match t {
                BlockTag::Latest => Some(self.client.info().best_hash),
                BlockTag::Pending => None,
            },
        }
        .ok_or("Failed to retrieve the substrate block id".to_string())
    }
}

/// Taken from https://github.com/paritytech/substrate/blob/master/client/rpc/src/author/mod.rs#L78
const TX_SOURCE: TransactionSource = TransactionSource::External;

#[async_trait]
#[allow(unused_variables)]
impl<B, BE, C, P, H> StarknetRpcApiServer for Starknet<B, BE, C, P, H>
where
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    H: HasherT + ThreadSafeCopy,
{
    fn block_number(&self) -> RpcResult<u64> {
        self.current_block_number()
    }

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

    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        Ok(block.header().transaction_count)
    }

    /// get the storage at a given address and key and at a given block
    fn get_storage_at(
        &self,
        contract_address: FieldElement,
        key: FieldElement,
        block_id: BlockId,
    ) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let runtime_api = self.client.runtime_api();
        let hex_address = contract_address.into();
        let hex_key = key.into();

        let value = runtime_api
            .get_storage_at(substrate_block_hash, hex_address, hex_key)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to get storage from contract: {:#?}", e);
                StarknetRpcApiError::ContractNotFound
            })?;
        let value = FieldElement::from_byte_slice_be(&<[u8; 32]>::from(value)).map_err(|e| {
            error!("Failed to get storage from contract: {:#?}", e);
            StarknetRpcApiError::InternalServerError
        })?;
        Ok(value)
    }

    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let runtime_api = self.client.runtime_api();

        let calldata = request.calldata.iter().map(|x| Felt252Wrapper::from(*x)).collect();

        let result = runtime_api
            .call(substrate_block_hash, request.contract_address.into(), request.entry_point_selector.into(), calldata)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;
        Ok(result.iter().map(|x| format!("{:#x}", x.0)).collect())
    }

    /// Get the contract class at a given contract address for a given block id
    fn get_class_at(&self, contract_address: FieldElement, block_id: BlockId) -> RpcResult<ContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = contract_address.into();
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
            StarknetRpcApiError::ContractNotFound
        })?)
    }

    /// Get the contract class hash in the given block for the contract deployed at the given
    /// address
    ///
    /// # Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag
    /// * `contract_address` - The address of the contract whose class hash will be returned
    ///
    /// # Returns
    ///
    /// * `class_hash` - The class hash of the given contract
    fn get_class_hash_at(&self, contract_address: FieldElement, block_id: BlockId) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let class_hash = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_hash_by_address(substrate_block_hash, contract_address.into())
            .ok_or_else(|| {
                error!("Failed to retrieve contract class hash at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;
        Ok(class_hash.into())
    }

    // Implementation of the `syncing` RPC Endpoint.
    // It's an async function because it uses `sync_service.best_seen_block()`.
    //
    // # Returns
    // * `Syncing` - An Enum that can be a `mc_rpc_core::SyncStatus` struct or a `Boolean`.
    async fn syncing(&self) -> RpcResult<SyncStatusType> {
        // obtain best seen (highest) block number
        match self.sync_service.best_seen_block().await {
            Ok(best_seen_block) => {
                let best_number = self.client.info().best_number;
                let highest_number = best_seen_block.unwrap_or(best_number);

                // get a starknet block from the starting substrate block number
                let starting_block = madara_backend_client::starknet_block_from_substrate_hash(
                    self.client.as_ref(),
                    &self.overrides,
                    self.starting_block,
                );

                // get a starknet block from the current substrate block number
                let current_block = madara_backend_client::starknet_block_from_substrate_hash(
                    self.client.as_ref(),
                    &self.overrides,
                    best_number,
                );

                // get a starknet block from the highest substrate block number
                let highest_block = madara_backend_client::starknet_block_from_substrate_hash(
                    self.client.as_ref(),
                    &self.overrides,
                    highest_number,
                );

                if starting_block.is_ok() && current_block.is_ok() && highest_block.is_ok() {
                    // Convert block numbers and hashes to the respective type required by the `syncing` endpoint.
                    let starting_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(self.starting_block);
                    let starting_block_hash = starting_block?.header().hash(*self.hasher).0;

                    let current_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(best_number);
                    let current_block_hash = current_block?.header().hash(*self.hasher).0;

                    let highest_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(highest_number);
                    let highest_block_hash = highest_block?.header().hash(*self.hasher).0;

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
    fn get_class(&self, block_id: BlockId, class_hash: FieldElement) -> RpcResult<ContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_by_class_hash(substrate_block_hash, class_hash.into())
            .ok_or_else(|| {
                error!("Failed to retrieve contract class from hash '{class_hash}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(to_rpc_contract_class(contract_class).map_err(|e| {
            error!("Failed to convert contract class from hash '{class_hash}' to RPC contract class: {e}");
            StarknetRpcApiError::ContractNotFound
        })?)
    }

    /// Returns the specified block with transaction hashes.
    fn get_block_with_tx_hashes(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxHashes> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        let transactions = block.transactions_hashes().into_iter().map(FieldElement::from).collect();
        let blockhash = block.header().hash(*self.hasher);
        let parent_blockhash = block.header().parent_block_hash;
        let block_with_tx_hashes = BlockWithTxHashes {
            transactions,
            // TODO: Status hardcoded, get status from block
            status: BlockStatus::AcceptedOnL2,
            block_hash: blockhash.into(),
            parent_hash: parent_blockhash.into(),
            block_number: block.header().block_number.as_u64(),
            new_root: block.header().global_state_root.into(),
            timestamp: block.header().block_timestamp,
            sequencer_address: block.header().sequencer_address.into(),
        };
        Ok(MaybePendingBlockWithTxHashes::Block(block_with_tx_hashes))
    }

    /// Get the nonce associated with the given address at the given block
    fn get_nonce(&self, contract_address: FieldElement, block_id: BlockId) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let nonce = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .nonce(substrate_block_hash, contract_address.into())
            .ok_or_else(|| {
                error!("Failed to get nonce at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        let nonce = FieldElement::from_byte_slice_be(&<[u8; 32]>::from(nonce)).map_err(|e| {
            error!("Failed to retrieve nonce at '{contract_address}': {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

        Ok(nonce)
    }

    /// Returns the chain id.
    fn chain_id(&self) -> RpcResult<String> {
        let hash = self.client.info().best_hash;
        let res = self.client.runtime_api().chain_id(hash).map_err(|_| {
            error!("fetch runtime chain id failed");
            StarknetRpcApiError::InternalServerError
        })?;
        Ok(format!("0x{:x}", res))
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
        let invoke_tx = to_invoke_tx(invoke_transaction)?;

        let transaction: MPTransaction = invoke_tx.into();
        let extrinsic = self
            .client
            .runtime_api()
            .convert_transaction(best_block_hash, transaction.clone(), TxType::Invoke)
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::ClassHashNotFound
            })?
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::ClassHashNotFound
            })?;

        self.pool.submit_one(&SPBlockId::hash(self.client.info().best_hash), TX_SOURCE, extrinsic).await.map_err(
            |e| {
                error!("Failed to submit extrinsic: {:?}", e);
                StarknetRpcApiError::ContractError
            },
        )?;

        Ok(InvokeTransactionResult { transaction_hash: transaction.hash.into() })
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

        let deploy_account_transaction = to_deploy_account_tx(deploy_account_transaction).map_err(|e| {
            error!("{e}");
            StarknetRpcApiError::InternalServerError
        })?;

        let transaction: MPTransaction = deploy_account_transaction.into();
        let extrinsic = self
            .client
            .runtime_api()
            .convert_transaction(best_block_hash, transaction.clone(), TxType::DeployAccount)
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })?;

        self.pool.submit_one(&SPBlockId::hash(best_block_hash), TX_SOURCE, extrinsic).await.map_err(|e| {
            error!("Failed to submit extrinsic: {:?}", e);
            StarknetRpcApiError::InternalServerError
        })?;

        Ok(DeployAccountTransactionResult {
            transaction_hash: transaction.hash.into(),
            contract_address: transaction.sender_address.into(),
        })
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
    async fn estimate_fee(&self, request: BroadcastedTransaction, block_id: BlockId) -> RpcResult<FeeEstimate> {
        // TODO:
        //      - modify BroadcastedTransaction to assert versions == "0x100000000000000000000000000000001"
        //      - to ensure broadcasted query signatures aren't valid on mainnet

        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let tx = to_tx(request)?;
        let (actual_fee, gas_usage) = self
            .client
            .runtime_api()
            .estimate_fee(substrate_block_hash, tx)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;

        Ok(FeeEstimate { gas_price: 0, gas_consumed: gas_usage, overall_fee: actual_fee })
    }

    // Returns the details of a transaction by a given block id and index
    fn get_transaction_by_block_id_and_index(&self, block_id: BlockId, index: usize) -> RpcResult<Transaction> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        let block_transactions = block.transactions();
        match block_transactions {
            BlockTransactions::Full(transactions) => {
                let transaction = transactions.get(index).ok_or(StarknetRpcApiError::InvalidTxnIndex)?;
                Ok(Transaction::try_from(transaction.clone()).map_err(|e| {
                    error!("{:?}", e);
                    StarknetRpcApiError::InternalServerError
                })?)
            }
            BlockTransactions::Hashes(_) => Err(StarknetRpcApiError::InvalidTxnIndex.into()),
        }
    }

    /// Get block information with full transactions given the block id
    fn get_block_with_txs(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxs> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        let transactions = match block.transactions() {
            BlockTransactions::Full(transactions) => transactions.to_vec(),
            BlockTransactions::Hashes(_) => vec![],
        };

        let block_with_txs = BlockWithTxs {
            // TODO: Get status from block
            status: BlockStatus::AcceptedOnL2,
            block_hash: block.header().hash(*self.hasher).into(),
            parent_hash: block.header().parent_block_hash.into(),
            block_number: block.header().block_number.as_u64(),
            new_root: block.header().global_state_root.into(),
            timestamp: block.header().block_timestamp,
            sequencer_address: block.header().sequencer_address.into(),
            transactions: transactions
                .into_iter()
                .map(Transaction::try_from)
                .collect::<Result<Vec<_>, RPCTransactionConversionError>>()
                .map_err(|e| {
                    error!("{:#?}", e);
                    StarknetRpcApiError::InternalServerError
                })?,
        };

        Ok(MaybePendingBlockWithTxs::Block(block_with_txs))
    }

    /// Get the information about the result of executing the requested block
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<StateUpdate> {
        todo!("Not implemented")
    }

    /// Returns the transactions in the transaction pool, recognized by this sequencer
    async fn pending_transactions(&self) -> RpcResult<Vec<Transaction>> {
        todo!("Not implemented")
    }

    /// Returns all events matching the given filter
    async fn get_events(
        &self,
        filter: EventFilter,
        continuation_token: Option<String>,
        chunk_size: u64,
    ) -> RpcResult<EventsPage> {
        todo!("Not implemented")
    }

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

        let declare_tx = to_declare_tx(declare_transaction).map_err(|e| {
            error!("{e}");
            StarknetRpcApiError::InternalServerError
        })?;

        let transaction: MPTransaction = declare_tx.into();
        let extrinsic = self
            .client
            .runtime_api()
            .convert_transaction(best_block_hash, transaction.clone(), TxType::Declare)
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to convert transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })?;

        self.pool.submit_one(&SPBlockId::hash(best_block_hash), TX_SOURCE, extrinsic).await.map_err(|e| {
            error!("Failed to submit extrinsic: {:?}", e);
            StarknetRpcApiError::InternalServerError
        })?;

        Ok(DeclareTransactionResult { transaction_hash: transaction.hash.into(), class_hash: FieldElement::ZERO })
    }
}
