use jsonrpsee::core::{async_trait, RpcResult as Result};
use std::{marker::PhantomData, sync::Arc};

use kaioshin_rpc_core::StarkNetRpc;
use kaioshin_runtime::opaque::Block;
use kaioshin_runtime::{AccountId, Balance, Index};
use kp_starknet::{BlockId, Block};

use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub struct StarkNet<C, P, B> {
    pub client: Arc<C>.
    pub pool: FullPool<P>
    _phdata: PhantomData<B>,
}

impl<C, P> StarkNet<C, P> {
    pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
        Self {
            client,
            pool,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl <C, P> StarkNetRpcServer<C, P> for StarkNet<C, P>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    pub fn get_block_with_tx_hashes(&self, block_id: BlockId) -> Result<Option<Block>> {
        self.get_block_with_tx_hashes(block_id).await
    }
}
