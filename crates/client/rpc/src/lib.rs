//! Staknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use jsonrpsee::core::RpcResult;
use log::error;
pub use mc_rpc_core::StarknetRpcApiServer;
use mc_rpc_core::{BlockHashAndNumber, BlockId as StarknetBlockId};
use mc_storage::OverrideHandle;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_runtime::testing::H256;
use sp_runtime::traits::Block as BlockT;

/// A Starknet RPC server for Madara
pub struct Starknet<B: BlockT, BE, C> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
    _marker: PhantomData<(B, BE)>,
}

impl<B: BlockT, BE, C> Starknet<B, BE, C> {
    pub fn new(client: Arc<C>, backend: Arc<mc_db::Backend<B>>, overrides: Arc<OverrideHandle<B>>) -> Self {
        Self { client, backend, overrides, _marker: PhantomData }
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
}

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

        Ok(block.header().transaction_count)
    }
}
