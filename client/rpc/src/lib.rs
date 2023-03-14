use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use std::{marker::PhantomData, sync::Arc};

use kaioshin_rpc_core::{types::*, StarkNetRpc};
use sc_transaction_pool::FullPool

pub struct StarkNetBlock<B: Block, C> {
    pub client: Arc<C>.
    pub pool: FullPool<Block, FullClient>
    _phdata: PhantomData<B>,
}

impl <B: Block, C> StarkNetBlock<B, C> {
    pub fn new(client: Arc<C>, Arc<C>) -> Self {
        Self {
            client,
            _phdata: Default::default(),
        }
    }
}
