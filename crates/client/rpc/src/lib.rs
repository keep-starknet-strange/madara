use std::sync::Arc;

pub mod api;
pub mod error;

use crate::api::StarknetRpcApiServer;
use crate::error::StarknetRpcApiError as SNError;
use jsonrpsee::core::Error;

use madara_runtime::opaque::Block;
use mp_starknet::starknet_block::block::Block as SNBlock;
use pallet_starknet::api::StarknetRuntimeApi;

use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::H256;

pub struct StarknetRpcServer<C, P> {
    pub client: Arc<C>,
    pub pool: Arc<P>,
    // _marker: std::marker::PhantomData<M>,
}

impl<C, P> StarknetRpcServer<C, P> {
    pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
        Self {
            client,
            pool,
            // _marker: Default::default(),
        }
    }
}

impl<C, P> StarknetRpcApiServer for StarknetRpcServer<C, P>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: StarknetRuntimeApi<Block>,
    P: TransactionPool + 'static,
{
    fn block_hash(&self) -> Result<Option<H256>, Error> {
        // call into the runtime via sp_api
        let api = self.client.runtime_api();
		let block_hash = self.client.info().best_hash;

        let block = api.current_block_hash(block_hash)
            .map_err(|_| Error::from(SNError::BlockNotFound));

        block
    }
}
