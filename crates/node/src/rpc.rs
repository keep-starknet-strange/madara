//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use madara_runtime::opaque::Block;
use madara_runtime::{AccountId, Balance, Index};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use pallet_starknet::api::StarknetRuntimeApi;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    C::Api: StarknetRuntimeApi<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use madara_rpc::{StarknetRpcServer, api::StarknetRpcApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool, deny_unsafe } = deps;

    module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    module.merge(StarknetRpcServer::new(client.clone(), pool.clone()).into_rpc())?;

    Ok(module)
}
