use std::sync::Arc;

pub mod api;
pub mod error;

use jsonrpsee::core::Error;
use madara_runtime::opaque::Block as BlockT;
use pallet_starknet::api::StarknetRuntimeApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

use crate::api::{BlockHashAndNumber, BlockNumber, StarknetRpcApiServer};
use crate::error::{internal_server_error, StarknetRpcApiError as SNError};

pub struct StarknetRpcServer<C, P> {
    pub client: Arc<C>,
    pub pool: Arc<P>,
}

impl<C, P> StarknetRpcServer<C, P> {
    pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
        Self { client, pool }
    }
}

impl<C, P> StarknetRpcApiServer for StarknetRpcServer<C, P>
where
    C: ProvideRuntimeApi<BlockT>,
    C: HeaderBackend<BlockT> + HeaderMetadata<BlockT, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: StarknetRuntimeApi<BlockT>,
    P: TransactionPool + 'static,
{
    fn block_number(&self) -> Result<BlockNumber, Error> {
        let api = self.client.runtime_api();
        let block_hash = self.client.info().best_hash;

        let block_number = api
            .current_block_number(block_hash)
            .map_err(internal_server_error)?
            .ok_or_else(|| Error::from(SNError::BlockNotFound))?;

        Ok(block_number)
    }

    fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, Error> {
        let api = self.client.runtime_api();
        let block_hash = self.client.info().best_hash;

        let block_number = api
            .current_block_number(block_hash)
            .map_err(internal_server_error)?
            .ok_or_else(|| Error::from(SNError::BlockNotFound))?;

        let block_hash = api
            .current_block_hash(block_hash)
            .map_err(internal_server_error)?
            .ok_or_else(|| Error::from(SNError::BlockNotFound))?;

        Ok(BlockHashAndNumber { block_number, block_hash })
    }
}
