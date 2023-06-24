//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

mod starknet;
use std::sync::Arc;

use futures::channel::mpsc;
use jsonrpsee::RpcModule;
use madara_runtime::opaque::Block;
use madara_runtime::{AccountId, Hash, Index};
use mc_transaction_pool::{ChainApi, Pool};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::{Backend, StorageProvider};
use sc_consensus_manual_seal::rpc::EngineCommand;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
pub use starknet::StarknetDeps;

/// Full client dependencies.
pub struct FullDeps<A: ChainApi, C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Extrinsic pool graph instance.
    pub graph: Arc<Pool<A>>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// Manual seal command sink
    pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
    /// Starknet dependencies
    pub starknet: StarknetDeps<C, Block>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<A, C, P, BE>(
    deps: FullDeps<A, C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    A: ChainApi<Block = Block> + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + StorageProvider<Block, BE> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_starknet::runtime_api::StarknetRuntimeApi<Block>
        + pallet_starknet::runtime_api::ConvertTransactionRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    BE: Backend<Block> + 'static,
{
    use mc_rpc::{Starknet, StarknetRpcApiServer};
    use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool, deny_unsafe, starknet: starknet_params, command_sink, graph } = deps;

    let hasher = client.runtime_api().get_hasher(client.info().best_hash)?.into();

    module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    module.merge(
        Starknet::new(
            client,
            starknet_params.madara_backend,
            starknet_params.overrides,
            pool,
            graph,
            starknet_params.sync_service,
            starknet_params.starting_block,
            hasher,
        )
        .into_rpc(),
    )?;

    if let Some(command_sink) = command_sink {
        module.merge(
            // We provide the rpc handler with the sending end of the channel to allow the rpc
            // send EngineCommands to the background block authorship task.
            ManualSeal::new(command_sink).into_rpc(),
        )?;
    }

    Ok(module)
}
