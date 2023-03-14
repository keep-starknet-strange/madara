use jsonrpsee::{core::RpcResult as Result, proc_macros::rpc};
use std::sync::Arc;

use kaioshin_rpc_core::StarkNetRpcServer;
use kaioshin_runtime::opaque::Block;
use kaioshin_runtime::{AccountId, Balance, Index};

use sp_core::U256;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub struct StarkNetImpl<C, P> {
    pub client: Arc<C>,
    pub pool: Arc<P>,
}

impl<C, P> StarkNetImpl<C, P> {
    pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
        Self {
            client,
            pool,
        }
    }
}

// #[async_trait]
impl<C, P> StarkNetRpcServer for StarkNetImpl<C, P>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    fn get_block_with_tx_hashes(&self, block_id: U256) -> Result<Option<Block>> {
        self.get_block_with_tx_hashes(block_id)
    }
}
