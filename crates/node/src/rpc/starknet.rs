use std::pin::Pin;
use std::sync::Arc;

use mc_db::Backend;
use mc_genesis_data_provider::GenesisProvider;
use mc_storage::OverrideHandle;
use sc_network_sync::SyncingService;
use sc_transaction_pool_api::{TransactionPool, TransactionStatusStreamFor};
use sp_api::BlockT;
use sp_runtime::traits::Header as HeaderT;
use starknet_api::core::ClassHash;
use starknet_api::transaction::TransactionHash;

type DeclareTransactionStatusStream<P> = (TransactionHash, ClassHash, Pin<Box<TransactionStatusStreamFor<P>>>);

/// Extra dependencies for Starknet compatibility.
pub struct StarknetDeps<C, G: GenesisProvider, B: BlockT, P: TransactionPool<Block = B>> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Madara Backend.
    pub madara_backend: Arc<Backend<B>>,
    /// Starknet data access overrides.
    pub overrides: Arc<OverrideHandle<B>>,
    /// The Substrate client sync service.
    pub sync_service: Arc<SyncingService<B>>,
    /// The starting block for the syncing.
    pub starting_block: <<B>::Header as HeaderT>::Number,
    /// The genesis state data provider
    pub genesis_provider: Arc<G>,
    /// The channel used to send newly declared contract classes data to the madara backed
    pub contract_class_data_tx: tokio::sync::mpsc::UnboundedSender<DeclareTransactionStatusStream<P>>,
}

impl<C, G: GenesisProvider, B: BlockT, P: TransactionPool<Block = B>> Clone for StarknetDeps<C, G, B, P> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            madara_backend: self.madara_backend.clone(),
            overrides: self.overrides.clone(),
            sync_service: self.sync_service.clone(),
            starting_block: self.starting_block,
            genesis_provider: self.genesis_provider.clone(),
            contract_class_data_tx: self.contract_class_data_tx.clone(),
        }
    }
}
