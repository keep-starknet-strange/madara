//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::sync::Arc;

use codec::{Decode, Encode};
use errors::StarknetRpcApiError;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use madara_runtime::UncheckedExtrinsic;
use mc_rpc_core::utils::{to_invoke_tx, to_rpc_contract_class};
pub use mc_rpc_core::StarknetRpcApiServer;
use mc_storage::OverrideHandle;
use mp_starknet::crypto::commitment::calculate_invoke_tx_hash;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sc_network_sync::SyncingService;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_core::{H256, U256};
use sp_runtime::generic::BlockId;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use starknet_core::types::FieldElement;
use starknet_providers::jsonrpc::models::{
    BlockHashAndNumber, BlockId as StarknetBlockId, BlockStatus, BlockTag, BlockWithTxHashes,
    BroadcastedInvokeTransaction, ContractClass, FunctionCall, InvokeTransactionResult, MaybePendingBlockWithTxHashes,
    SyncStatus, SyncStatusType,
};

/// A Starknet RPC server for Madara
pub struct Starknet<B: BlockT, BE, C, P> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
    pool: Arc<P>,
    sync_service: Arc<SyncingService<B>>,
    starting_block: <<B>::Header as HeaderT>::Number,
    _marker: PhantomData<(B, BE)>,
}

/// Constructor for A Starknet RPC server for Madara
/// # Arguments
// * `client` - The Madara client
// * `backend` - The Madara backend
// * `overrides` - The OverrideHandle
// * `sync_service` - The Substrate client sync service
// * `starting_block` - The starting block for the syncing
//
// # Returns
// * `Self` - The actual Starknet struct
impl<B: BlockT, BE, C, P> Starknet<B, BE, C, P> {
    pub fn new(
        client: Arc<C>,
        backend: Arc<mc_db::Backend<B>>,
        overrides: Arc<OverrideHandle<B>>,
        pool: Arc<P>,
        sync_service: Arc<SyncingService<B>>,
        starting_block: <<B>::Header as HeaderT>::Number,
    ) -> Self {
        Self { client, backend, overrides, pool, sync_service, starting_block, _marker: PhantomData }
    }
}

impl<B, BE, C, P> Starknet<B, BE, C, P>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, BE, C, P> Starknet<B, BE, C, P>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    BE: Backend<B>,
{
    pub fn current_block_hash(&self) -> Result<H256, ApiError> {
        let substrate_block_hash = self.client.info().best_hash;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        Ok(block.header().hash())
    }

    /// Returns the substrate block corresponding to the given Starknet block id
    fn substrate_block_hash_from_starknet_block(&self, block_id: StarknetBlockId) -> Result<B::Hash, String> {
        match block_id {
            StarknetBlockId::Hash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_slice(&h.to_bytes_be()[..32]),
            )
            .map_err(|e| format!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}"))?,
            StarknetBlockId::Number(n) => self
                .client
                .hash(UniqueSaturatedInto::unique_saturated_into(n))
                .map_err(|e| format!("Failed to retrieve the hash of block number '{n}': {e}"))?,
            StarknetBlockId::Tag(t) => match t {
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
impl<B, BE, C, P> StarknetRpcApiServer for Starknet<B, BE, C, P>
where
    B: BlockT,
    P: TransactionPool<Block = B> + 'static,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
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

        // TODO: fix block hash, see https://github.com/keep-starknet-strange/madara/issues/381
        Ok(BlockHashAndNumber {
            block_hash: FieldElement::from_byte_slice_be(&block_hash.as_bytes()[..31]).unwrap(),
            block_number,
        })
    }

    fn get_block_transaction_count(&self, block_id: StarknetBlockId) -> RpcResult<u128> {
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
        block_id: StarknetBlockId,
    ) -> RpcResult<FieldElement> {
        let substrate_block_hash = match block_id {
            StarknetBlockId::Hash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_slice(&h.to_bytes_be()[..32]),
            )
            .map_err(|e| {
                error!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}");
                StarknetRpcApiError::BlockNotFound
            })?,
            StarknetBlockId::Number(n) => {
                self.client.hash(UniqueSaturatedInto::unique_saturated_into(n)).map_err(|e| {
                    error!("Failed to retrieve the hash of block number '{n}': {e}");
                    StarknetRpcApiError::BlockNotFound
                })?
            }
            StarknetBlockId::Tag(t) => match t {
                BlockTag::Latest => Some(self.client.info().best_hash),
                BlockTag::Pending => None,
            },
        }
        .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let runtime_api = self.client.runtime_api();
        let hex_address = contract_address.to_bytes_be();
        let hex_key = H256::from_slice(&key.to_bytes_be()[..32]);

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
        let value = FieldElement::from_byte_slice_be(&<[u8; 32]>::from(value)[..31]).map_err(|e| {
            error!("Failed to get storage from contract: {:#?}", e);
            StarknetRpcApiError::InternalServerError
        })?;
        Ok(value)
    }

    fn call(&self, request: FunctionCall, block_id: StarknetBlockId) -> RpcResult<Vec<String>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let runtime_api = self.client.runtime_api();

        let calldata = request.calldata.iter().map(|x| U256::from(x.to_bytes_be())).collect();

        let result = runtime_api
            .call(
                substrate_block_hash,
                request.contract_address.to_bytes_be(),
                H256::from_slice(&request.entry_point_selector.to_bytes_be()[..32]),
                calldata,
            )
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;
        Ok(result.iter().map(|x| format!("{:#x}", x)).collect())
    }

    /// Get the contract class at a given contract address for a given block id
    fn get_class_at(&self, contract_address: FieldElement, block_id: StarknetBlockId) -> RpcResult<ContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = contract_address.to_bytes_be();
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
                    let starting_block_hash =
                        FieldElement::from_byte_slice_be(&starting_block?.header().hash().to_fixed_bytes()[..31])
                            .unwrap();
                    let current_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(best_number);
                    let current_block_hash =
                        FieldElement::from_byte_slice_be(&current_block?.header().hash().to_fixed_bytes()[..31])
                            .unwrap();
                    let highest_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(highest_number);
                    let highest_block_hash =
                        FieldElement::from_byte_slice_be(&highest_block?.header().hash().to_fixed_bytes()[..31])
                            .unwrap();

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
    fn get_class(&self, block_id: StarknetBlockId, class_hash: FieldElement) -> RpcResult<ContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_clash_hashed_wrapped = class_hash.to_bytes_be();

        let contract_class = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_by_class_hash(substrate_block_hash, contract_clash_hashed_wrapped)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class from hash '{class_hash}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(to_rpc_contract_class(contract_class).map_err(|e| {
            error!("Failed to convert contract class from hash '{class_hash}' to RPC contract class: {e}");
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
    fn get_class_hash_at(&self, contract_address: FieldElement, block_id: StarknetBlockId) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = contract_address.to_bytes_be();

        let class_hash = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_hash_by_address(substrate_block_hash, contract_address_wrapped)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class hash at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;
        let class_hash = FieldElement::from_byte_slice_be(&class_hash[..31]).unwrap();
        Ok(class_hash)
    }

    /// Returns the specified block with transaction hashes.
    fn get_block_with_tx_hashes(&self, block_id: StarknetBlockId) -> RpcResult<MaybePendingBlockWithTxHashes> {
        let mut block_status = BlockStatus::AcceptedOnL2;
        let substrate_block_hash = match block_id {
            StarknetBlockId::Hash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_slice(&h.to_bytes_be()[..32]),
            )
            .map_err(|e| {
                error!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}");
                StarknetRpcApiError::BlockNotFound
            })?,
            StarknetBlockId::Number(n) => {
                self.client.hash(UniqueSaturatedInto::unique_saturated_into(n)).map_err(|e| {
                    error!("Failed to retrieve the hash of block number '{n}': {e}");
                    StarknetRpcApiError::BlockNotFound
                })?
            }
            StarknetBlockId::Tag(t) => match t {
                BlockTag::Latest => Some(self.client.info().best_hash),
                BlockTag::Pending => {
                    block_status = BlockStatus::Pending;
                    None
                }
            },
        }
        .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        let transactions = block
            .transactions_hashes()
            .into_iter()
            .map(|hash| FieldElement::from_byte_slice_be(&hash.to_fixed_bytes()[..31]).unwrap())
            .collect();
        let block_with_tx_hashes = BlockWithTxHashes {
            transactions,
            // TODO: Status hardcoded, get status from block
            status: block_status,
            block_hash: FieldElement::from_byte_slice_be(&block.header().hash().to_fixed_bytes()[..31]).unwrap(),
            parent_hash: FieldElement::from_byte_slice_be(&block.header().parent_block_hash.to_fixed_bytes()[..31])
                .unwrap(),
            block_number: block.header().block_number.as_u64(),
            new_root: FieldElement::from_byte_slice_be(&<[u8; 32]>::from(block.header().global_state_root)[..31])
                .unwrap(),
            timestamp: block.header().block_timestamp,
            sequencer_address: FieldElement::from_byte_slice_be(&block.header().sequencer_address[..31]).unwrap(),
        };
        Ok(MaybePendingBlockWithTxHashes::Block(block_with_tx_hashes))
    }

    /// Returns the chain id.
    fn get_chain_id(&self) -> RpcResult<String> {
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
    /// * `invoke tx` - https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#invoke_transaction
    ///
    /// # Returns
    ///
    /// * `transaction_hash` - transaction hash corresponding to the invocation
    async fn add_invoke_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTransaction,
    ) -> RpcResult<InvokeTransactionResult> {
        let invoke_tx = to_invoke_tx(invoke_transaction)?;

        let call = pallet_starknet::Call::invoke { transaction: invoke_tx.clone() };
        let extrinsic = UncheckedExtrinsic::new_unsigned(call.into());
        let encoded_entrinsic = Encode::encode(&extrinsic);
        let extrinsic = match Decode::decode(&mut &encoded_entrinsic[..]) {
            Ok(xt) => xt,
            Err(err) => {
                error!("Failed to decode extrinsic: {:?}", err);
                return Err(StarknetRpcApiError::InternalServerError.into());
            }
        };

        self.pool.submit_one(&BlockId::hash(self.client.info().best_hash), TX_SOURCE, extrinsic).await.map_err(
            |e| {
                error!("Failed to submit extrinsic: {:?}", e);
                StarknetRpcApiError::ContractError
            },
        )?;

        let invoke_tx_hash = calculate_invoke_tx_hash(invoke_tx);
        Ok(InvokeTransactionResult {
            transaction_hash: starknet_ff::FieldElement::from_bytes_be(invoke_tx_hash.as_fixed_bytes()).unwrap(),
        })
    }
}
