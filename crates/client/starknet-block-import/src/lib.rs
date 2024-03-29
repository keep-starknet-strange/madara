//! Starknet block import.
//!
//! This crate introduces a Starknet specific handler that can be added to the block import
//! pipeline. More specifically, it "wraps" the underlying block import logic i.e. executes first in
//! the queue.
//!
//! Read more about block import pipeline:
//!   * https://docs.substrate.io/learn/transaction-lifecycle/#block-authoring-and-block-imports
//!   * https://doc.deepernetwork.org/v3/advanced/block-import/
//!
//! The purpose of this custom block import logic is to do additional checks for declare
//! transactions. Despite the introduction of the safe intermediate representation (Sierra)
//! for Cairo, there's still a possibility of the following attack:
//!   - User crafts a malicious Cairo contract the execution of which cannot be proven
//!   - User submits that transaction via a patched node that does not verify Sierra classes
//!   - The runtime does not have Sierra classes and therefore cannot check the validiy either
//!   - Sequencer fails to prove the execution and also cannot charge for the work done => DoS
//!     attack vector
//!
//! Read more about Sierra and the problem it addresses:
//!   * https://docs.starknet.io/documentation/architecture_and_concepts/Smart_Contracts/cairo-and-sierra/
//!
//! Starknet block import solves the issue above as follows:
//!   - Upon receiving a new block it searches for declare transactions
//!   - For every declare transaction found, it tries to find according Sierra classes in the local
//!     DB
//!   - It tries to compile the class to check if they match the contract class from the transaction
//!   - The block import fails if there is at least one transaction with mismatching class hashes
//!
//! NOTES:
//!
//! Currently Sierra classes DB is populated when user submits RPC requests and therefore the
//! approch works for single node setup only.
//! In order to make it work in the multi-node setting, one needs to query missing Sierra classes
//! from other nodes via P2P (see https://github.com/starknet-io/starknet-p2p-specs/blob/main/p2p/proto/class.proto)
//!
//! Cairo compiler version mismatch can be a problem, e.g. if the version used by Madara is lagging
//! behind significantly it can fail to compile Sierra classes obtained from the recent compilers.
//! Similarly, old Sierra classes might not compile because of the broken backward compatibility.

use std::sync::Arc;

use async_trait::async_trait;
use madara_runtime::opaque::Block;
use madara_runtime::Hash;
use mp_transactions::{DeclareTransaction, Transaction};
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult, JustificationImport};
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::NumberFor;
use sp_runtime::Justification;

mod compilation;
mod validation;

use crate::validation::validate_declare_v2_transaction;

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
        log::info!("🐺 Starknet block import: verifying declared CASM classes against local Sierra classes");
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
                if let Transaction::Declare(DeclareTransaction::V2(declare_v2), casm_class) = tx {
                    validate_declare_v2_transaction(
                        declare_v2,
                        casm_class,
                        self.madara_backend.sierra_classes().clone(),
                    )?;
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

    pub fn inner(&self) -> &I {
        &self.inner
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
