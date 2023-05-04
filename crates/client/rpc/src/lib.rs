//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use hex::FromHex;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
pub use mc_rpc_core::StarknetRpcApiServer;
use mc_rpc_core::{
    to_rpc_contract_class, BlockHashAndNumber, BlockId as StarknetBlockId, BlockStatus, BlockWithTxHashes,
    ContractAddress, ContractClassHash, FunctionCall, MaybePendingBlockWithTxHashes, RPCContractClass, Syncing,
};
use mc_storage::OverrideHandle;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sc_network_sync::SyncingService;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_core::{H256, U256};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use starknet_api::hash::StarkFelt;

/// A Starknet RPC server for Madara
pub struct Starknet<B: BlockT, BE, C> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
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
impl<B: BlockT, BE, C> Starknet<B, BE, C> {
    pub fn new(
        client: Arc<C>,
        backend: Arc<mc_db::Backend<B>>,
        overrides: Arc<OverrideHandle<B>>,
        sync_service: Arc<SyncingService<B>>,
        starting_block: <<B>::Header as HeaderT>::Number,
    ) -> Self {
        Self { client, backend, overrides, sync_service, starting_block, _marker: PhantomData }
    }
}

impl<B, BE, C> Starknet<B, BE, C>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, BE, C> Starknet<B, BE, C>
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
            StarknetBlockId::BlockHash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_str(&h).map_err(|e| format!("Failed to convert '{h}' to H256: {e}"))?,
            )
            .map_err(|e| format!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}"))?,
            StarknetBlockId::BlockNumber(n) => self
                .client
                .hash(UniqueSaturatedInto::unique_saturated_into(n))
                .map_err(|e| format!("Failed to retrieve the hash of block number '{n}': {e}"))?,
            StarknetBlockId::BlockTag(t) => match t {
                mc_rpc_core::BlockTag::Latest => Some(self.client.info().best_hash),
                mc_rpc_core::BlockTag::Pending => None,
            },
        }
        .ok_or("Failed to retrieve the substrate block id".to_string())
    }
}

#[async_trait]
impl<B, BE, C> StarknetRpcApiServer for Starknet<B, BE, C>
where
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    fn block_number(&self) -> RpcResult<mc_rpc_core::BlockNumber> {
        self.current_block_number()
    }

    fn block_hash_and_number(&self) -> RpcResult<mc_rpc_core::BlockHashAndNumber> {
        let block_number = self.current_block_number()?;
        let block_hash = self.current_block_hash().map_err(|e| {
            error!("Failed to retrieve the current block hash: {}", e);
            StarknetRpcApiError::NoBlocks
        })?;

        Ok(BlockHashAndNumber { block_hash: format!("{:#x}", block_hash), block_number })
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

    fn call(&self, request: FunctionCall, block_id: StarknetBlockId) -> RpcResult<Vec<String>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let runtime_api = self.client.runtime_api();

        let calldata: Result<Vec<U256>, StarknetRpcApiError> = request
            .calldata
            .iter()
            .map(|x| {
                U256::from_str(x).map_err(|e| {
                    error!("Calldata: Failed to convert '{x}' to U256: {e}");
                    StarknetRpcApiError::InvalidCallData
                })
            })
            .collect();

        match calldata {
            Ok(calldata) => {
                let result = runtime_api
                    .call(
                        substrate_block_hash,
                        <[u8; 32]>::from_hex(remove_prefix(&request.contract_address)).map_err(|e| {
                            error!("Address: Failed to convert '{0}' to [u8; 32]: {e}", request.contract_address);
                            StarknetRpcApiError::BlockNotFound
                        })?,
                        H256::from_str(&request.entry_point_selector).map_err(|e| {
                            error!("Entrypoint: Failed to convert '{0}' to H256: {e}", request.entry_point_selector);
                            StarknetRpcApiError::BlockNotFound
                        })?,
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
            Err(e) => Err(e.into()),
        }
    }

    /// Get the contract class at a given contract address for a given block id
    fn get_class_at(
        &self,
        contract_address: ContractAddress,
        block_id: StarknetBlockId,
    ) -> RpcResult<RPCContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = <[u8; 32]>::from_hex(remove_prefix(&contract_address)).map_err(|e| {
            error!("Failed to convert '{contract_address}' to array: {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

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
    async fn syncing(&self) -> RpcResult<Syncing> {
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
                    let starting_block_hash = format!("{:#x}", starting_block?.header().hash());
                    let current_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(best_number);
                    let current_block_hash = format!("{:#x}", current_block?.header().hash());
                    let highest_block_num = UniqueSaturatedInto::<u64>::unique_saturated_into(highest_number);
                    let highest_block_hash = format!("{:#x}", highest_block?.header().hash());

                    // Build the `SyncStatus` struct with the respective syn information
                    Ok(Syncing::SyncStatus(mc_rpc_core::SyncStatus {
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
                    Ok(Syncing::False(false))
                }
            }
            Err(_) => {
                // If there was an error when getting a starknet block, then we return `false`,
                // as per the endpoint specification
                log::error!("`SyncingEngine` shut down");
                Ok(Syncing::False(false))
            }
        }
    }

    /// Get the contract class definition in the given block associated with the given hash.
    fn get_class(&self, block_id: StarknetBlockId, class_hash: ContractClassHash) -> RpcResult<RPCContractClass> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_clash_hashed_wrapped = <[u8; 32]>::from_hex(remove_prefix(&class_hash)).map_err(|e| {
            error!("Failed to convert '{class_hash}' to array: {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

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
    fn get_class_hash_at(
        &self,
        contract_address: ContractAddress,
        block_id: StarknetBlockId,
    ) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = <[u8; 32]>::from_hex(remove_prefix(&contract_address)).map_err(|e| {
            error!("Failed to convert '{contract_address}' to array: {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

        let class_hash = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_hash_by_address(substrate_block_hash, contract_address_wrapped)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class hash at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(StarkFelt::new(class_hash)
            .map_err(|e| {
                error!("Failed to convert contract class hash at '{contract_address}': {e}");
                StarknetRpcApiError::ContractNotFound
            })?
            .to_string())
    }
    fn get_block_with_tx_hashes(&self, block_id: StarknetBlockId) -> RpcResult<MaybePendingBlockWithTxHashes> {
        println!("StarknetBlockId {:?}", block_id);
        let substrate_block_hash = match block_id {
            StarknetBlockId::BlockHash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_str(&h).map_err(|e| {
                    error!("Failed to convert '{h}' to H256: {e}");
                    StarknetRpcApiError::BlockNotFound
                })?,
            )
            .map_err(|e| {
                error!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}");
                StarknetRpcApiError::BlockNotFound
            })?,
            StarknetBlockId::BlockNumber(n) => {
                self.client.hash(UniqueSaturatedInto::unique_saturated_into(n)).map_err(|e| {
                    error!("Failed to retrieve the hash of block number '{n}': {e}");
                    StarknetRpcApiError::BlockNotFound
                })?
            }
            StarknetBlockId::BlockTag(t) => match t {
                mc_rpc_core::BlockTag::Latest => Some(self.client.info().best_hash),
                mc_rpc_core::BlockTag::Pending => None,
            },
        }
        .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        let transaction_hashes = block.transactions_hashes().into_iter().map(|hash| hash.to_string()).collect();
        let block_with_tx_hashes = BlockWithTxHashes {
            transactions: transaction_hashes,
            status: BlockStatus::Pending, // TODO: get real value
            block_hash: block.header().hash().to_string(),
            // parent_hash: FieldElement::from("0x0"),
            parent_hash: block.header().parent_block_hash.to_string(),
            block_number: UniqueSaturatedInto::<u64>::unique_saturated_into(block.header().block_number),
            new_root: block.header().global_state_root.to_string(),
            // new_root: FieldElement::from("0x0"),
            timestamp: block.header().block_timestamp,
            sequencer_address: H256::from_slice(&block.header().sequencer_address).to_string(),
        };
        Ok(MaybePendingBlockWithTxHashes::Block(block_with_tx_hashes))
    }
}

/// Removes the "0x" prefix from a given hexadecimal string
fn remove_prefix(input: &str) -> &str {
    input.strip_prefix("0x").unwrap_or(input)
}
