#![doc = include_str!("../README.md")]

use std::sync::Arc;

use async_trait::async_trait;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use madara_runtime::opaque::Block;
use madara_runtime::Hash;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult, JustificationImport};
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::NumberFor;
use sp_runtime::Justification;

mod compilation;
mod validation;

use crate::validation::validate_declare_transaction;

type MadaraBackend = mc_db::Backend<Block>;

pub struct StarknetBlockImport<I: Clone, C: ProvideRuntimeApi<Block>> {
    inner: I,
    client: Arc<C>,
    madara_backend: Arc<MadaraBackend>,
}

#[async_trait]
impl<I, C> BlockImport<Block> for StarknetBlockImport<I, C>
where
    I: BlockImport<Block, Error = ConsensusError> + Send + Clone,
    C: ProvideRuntimeApi<Block> + Send + Sync,
    C::Api: StarknetRuntimeApi<Block>,
{
    type Error = ConsensusError;

    async fn check_block(&mut self, block: BlockCheckParams<Block>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(&mut self, block: BlockImportParams<Block>) -> Result<ImportResult, Self::Error> {
        log::debug!("🐺 Starknet block import: verifying declared CASM classes against local Sierra classes");
        if let Some(extrinsics) = &block.body {
            // Extrinsic filter does not access the block state so technically the block hash does not matter.
            // But since we need to provide one anyways, parent hash is a convenient option.
            let prev_block_hash = *block.header.parent_hash();
            let transactions: Vec<Transaction> = self
                .client
                .runtime_api()
                .extrinsic_filter(prev_block_hash, extrinsics.clone())
                .map_err(|e| ConsensusError::ClientImport(e.to_string()))?;

            for tx in transactions {
                if let Transaction::AccountTransaction(AccountTransaction::Declare(declare)) = tx {
                    log::trace!("🐺 Starknet block import: checking declare transaction\n\t{:?}", declare,);
                    validate_declare_transaction(declare, self.madara_backend.sierra_classes().clone())?;
                }
            }
        }

        self.inner.import_block(block).await
    }
}

#[async_trait]
impl<I, C> JustificationImport<Block> for StarknetBlockImport<I, C>
where
    I: JustificationImport<Block> + Send + Clone,
    C: ProvideRuntimeApi<Block> + Send + Sync,
{
    type Error = I::Error;

    async fn on_start(&mut self) -> Vec<(Hash, NumberFor<Block>)> {
        self.inner.on_start().await
    }

    async fn import_justification(
        &mut self,
        hash: Hash,
        number: NumberFor<Block>,
        justification: Justification,
    ) -> Result<(), Self::Error> {
        self.inner.import_justification(hash, number, justification).await
    }
}

impl<I, C> StarknetBlockImport<I, C>
where
    I: BlockImport<Block> + Send + Sync + Clone,
    C: ProvideRuntimeApi<Block> + Send,
{
    pub fn new(inner: I, client: Arc<C>, madara_backend: Arc<MadaraBackend>) -> Self {
        Self { inner, client, madara_backend }
    }

    pub fn unwrap(self) -> I {
        self.inner
    }
}

// https://github.com/rust-lang/rust/issues/41481
impl<I, C> Clone for StarknetBlockImport<I, C>
where
    I: BlockImport<Block> + Send + Clone,
    C: ProvideRuntimeApi<Block> + Send,
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), client: self.client.clone(), madara_backend: self.madara_backend.clone() }
    }
}
