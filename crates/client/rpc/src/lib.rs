mod errors;

use std::marker::PhantomData;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use madara_rpc_core::{BlockHashAndNumber, StarknetRpcApiServer};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_runtime::testing::H256;
use sp_runtime::traits::Block;

pub struct Starknet<B: Block, C> {
    client: Arc<C>,
    _marker: PhantomData<B>,
}

impl<B, C> Starknet<B, C>
where
    B: Block,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, C> Starknet<B, C>
where
    B: Block,
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

#[async_trait]
impl<B, C> StarknetRpcApiServer for Starknet<B, C>
where
    B: Block,
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

        Ok(BlockHashAndNumber { block_hash: block_hash.to_string(), block_number })
    }
}

pub mod starknet_backend_client {}
