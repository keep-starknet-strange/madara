mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use jsonrpsee::core::RpcResult;
use log::error;
pub use madara_rpc_core::StarknetRpcApiServer;
use madara_rpc_core::{BlockHashAndNumber, BlockId as StarknetBlockId};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_runtime::testing::H256;
use sp_runtime::traits::Block as BlockT;

pub struct Starknet<B: BlockT, C> {
    client: Arc<C>,
    backend: Arc<madara_db::Backend<B>>,
    _marker: PhantomData<B>,
}

impl<B: BlockT, C> Starknet<B, C> {
    pub fn new(client: Arc<C>, backend: Arc<madara_db::Backend<B>>) -> Self {
        Self { client, backend, _marker: PhantomData }
    }
}

impl<B, C> Starknet<B, C>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, C> Starknet<B, C>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    pub fn current_block_hash(&self) -> Result<H256, ApiError> {
        let substrate_block_hash = self.client.info().best_hash;

        let api = self.client.runtime_api();

        let block_hash = api.current_block_hash(substrate_block_hash)?;

        Ok(block_hash)
    }
}

impl<B, C> StarknetRpcApiServer for Starknet<B, C>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    fn block_number(&self) -> RpcResult<madara_rpc_core::BlockNumber> {
        self.current_block_number()
    }

    fn block_hash_and_number(&self) -> RpcResult<madara_rpc_core::BlockHashAndNumber> {
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
                madara_rpc_core::BlockTag::Latest => Some(self.client.info().best_hash),
                madara_rpc_core::BlockTag::Pending => None,
            },
        }
        .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let api = self.client.runtime_api();

        let block = api.current_block(substrate_block_hash).map_err(|e| {
            error!(
                "Failed retrieve Starknet block using the Starknet pallet runtime API for Substrate block with hash \
                 '{substrate_block_hash}': {e}"
            );
            StarknetRpcApiError::BlockNotFound
        })?;

        Ok(block.header.transaction_count)
    }
}
