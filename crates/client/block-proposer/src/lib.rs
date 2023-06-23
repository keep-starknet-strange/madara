//! Block proposer implementation.
//! This crate implements the [`sp_consensus::Proposer`] trait.
//! It is used to build blocks for the block authoring node.
//! The block authoring node is the node that is responsible for building new blocks.
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time;

use codec::Encode;
use futures::channel::oneshot;
use futures::future::{Future, FutureExt};
use futures::{future, select};
use log::{debug, error, info, trace, warn};
use prometheus_endpoint::Registry as PrometheusRegistry;
use sc_block_builder::{BlockBuilderApi, BlockBuilderProvider};
use sc_client_api::backend;
use sc_proposer_metrics::{EndProposingReason, MetricsLink as PrometheusMetrics};
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::ApplyExtrinsicFailed::Validity;
use sp_blockchain::Error::ApplyExtrinsicFailed;
use sp_blockchain::HeaderBackend;
use sp_consensus::{DisableProofRecording, ProofRecording, Proposal};
use sp_core::traits::SpawnNamed;
use sp_inherents::InherentData;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::{Digest, Percent, SaturatedConversion};

/// Default block size limit in bytes used by [`Proposer`].
///
/// Can be overwritten by [`ProposerFactory::set_default_block_size_limit`].
///
/// Be aware that there is also an upper packet size on what the networking code
/// will accept. If the block doesn't fit in such a package, it can not be
/// transferred to other nodes.
pub const DEFAULT_BLOCK_SIZE_LIMIT: usize = 4 * 1024 * 1024 + 512;
/// Default value for `soft_deadline_percent` used by [`Proposer`].
/// `soft_deadline_percent` value is used to compute soft deadline during block production.
/// The soft deadline indicates where we should stop attempting to add transactions
/// to the block, which exhaust resources. After soft deadline is reached,
/// we switch to a fixed-amount mode, in which after we see `MAX_SKIPPED_TRANSACTIONS`
/// transactions which exhaust resources, we will conclude that the block is full.
const DEFAULT_SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(80);

const LOG_TARGET: &str = "block-proposer";

/// [`Proposer`] factory.
pub struct ProposerFactory<A, B, C, PR> {
    spawn_handle: Box<dyn SpawnNamed>,
    /// The client instance.
    client: Arc<C>,
    /// The transaction pool.
    transaction_pool: Arc<A>,
    /// Prometheus Link,
    metrics: PrometheusMetrics,
    /// The default block size limit.
    ///
    /// If no `block_size_limit` is passed to [`sp_consensus::Proposer::propose`], this block size
    /// limit will be used.
    default_block_size_limit: usize,
    /// Soft deadline percentage of hard deadline.
    ///
    /// The value is used to compute soft deadline during block production.
    /// The soft deadline indicates where we should stop attempting to add transactions
    /// to the block, which exhaust resources. After soft deadline is reached,
    /// we switch to a fixed-amount mode, in which after we see `MAX_SKIPPED_TRANSACTIONS`
    /// transactions which exhaust resources, we will conclude that the block is full.
    soft_deadline_percent: Percent,
    /// phantom member to pin the `Backend`/`ProofRecording` type.
    _phantom: PhantomData<(B, PR)>,
}

impl<A, B, C> ProposerFactory<A, B, C, DisableProofRecording> {
    /// Create a new proposer factory.
    ///
    /// Proof recording will be disabled when using proposers built by this instance to build
    /// blocks.
    pub fn new(
        spawn_handle: impl SpawnNamed + 'static,
        client: Arc<C>,
        transaction_pool: Arc<A>,
        prometheus: Option<&PrometheusRegistry>,
    ) -> Self {
        ProposerFactory {
            spawn_handle: Box::new(spawn_handle),
            transaction_pool,
            metrics: PrometheusMetrics::new(prometheus),
            default_block_size_limit: DEFAULT_BLOCK_SIZE_LIMIT,
            soft_deadline_percent: DEFAULT_SOFT_DEADLINE_PERCENT,
            client,
            _phantom: PhantomData,
        }
    }
}

impl<A, B, C, PR> ProposerFactory<A, B, C, PR> {
    /// Set the default block size limit in bytes.
    ///
    /// The default value for the block size limit is:
    /// [`DEFAULT_BLOCK_SIZE_LIMIT`].
    ///
    /// If there is no block size limit passed to [`sp_consensus::Proposer::propose`], this value
    /// will be used.
    pub fn set_default_block_size_limit(&mut self, limit: usize) {
        self.default_block_size_limit = limit;
    }

    /// Set soft deadline percentage.
    ///
    /// The value is used to compute soft deadline during block production.
    /// The soft deadline indicates where we should stop attempting to add transactions
    /// to the block, which exhaust resources. After soft deadline is reached,
    /// we switch to a fixed-amount mode, in which after we see `MAX_SKIPPED_TRANSACTIONS`
    /// transactions which exhaust resources, we will conclude that the block is full.
    ///
    /// Setting the value too low will significantly limit the amount of transactions
    /// we try in case they exhaust resources. Setting the value too high can
    /// potentially open a DoS vector, where many "exhaust resources" transactions
    /// are being tried with no success, hence block producer ends up creating an empty block.
    pub fn set_soft_deadline(&mut self, percent: Percent) {
        self.soft_deadline_percent = percent;
    }
}

impl<B, Block, C, A, PR> ProposerFactory<A, B, C, PR>
where
    A: TransactionPool<Block = Block> + 'static,
    B: backend::Backend<Block> + Send + Sync + 'static,
    Block: BlockT,
    C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>> + BlockBuilderApi<Block>,
{
    fn init_with_now(
        &mut self,
        parent_header: &<Block as BlockT>::Header,
        now: Box<dyn Fn() -> time::Instant + Send + Sync>,
    ) -> Proposer<B, Block, C, A, PR> {
        let parent_hash = parent_header.hash();

        info!("🩸 Starting consensus session on top of parent {:?}", parent_hash);

        let proposer = Proposer::<_, _, _, _, PR> {
            spawn_handle: self.spawn_handle.clone(),
            client: self.client.clone(),
            parent_hash,
            parent_number: *parent_header.number(),
            transaction_pool: self.transaction_pool.clone(),
            now,
            metrics: self.metrics.clone(),
            default_block_size_limit: self.default_block_size_limit,
            soft_deadline_percent: self.soft_deadline_percent,
            _phantom: PhantomData,
        };

        proposer
    }
}

impl<A, B, Block, C, PR> sp_consensus::Environment<Block> for ProposerFactory<A, B, C, PR>
where
    A: TransactionPool<Block = Block> + 'static,
    B: backend::Backend<Block> + Send + Sync + 'static,
    Block: BlockT,
    C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>> + BlockBuilderApi<Block>,
    PR: ProofRecording,
{
    type CreateProposer = future::Ready<Result<Self::Proposer, Self::Error>>;
    type Proposer = Proposer<B, Block, C, A, PR>;
    type Error = sp_blockchain::Error;

    fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> Self::CreateProposer {
        future::ready(Ok(self.init_with_now(parent_header, Box::new(time::Instant::now))))
    }
}

/// The proposer logic.
pub struct Proposer<B, Block: BlockT, C, A: TransactionPool, PR> {
    spawn_handle: Box<dyn SpawnNamed>,
    client: Arc<C>,
    parent_hash: Block::Hash,
    parent_number: <<Block as BlockT>::Header as HeaderT>::Number,
    transaction_pool: Arc<A>,
    now: Box<dyn Fn() -> time::Instant + Send + Sync>,
    metrics: PrometheusMetrics,
    default_block_size_limit: usize,
    soft_deadline_percent: Percent,
    _phantom: PhantomData<(B, PR)>,
}

impl<A, B, Block, C, PR> sp_consensus::Proposer<Block> for Proposer<B, Block, C, A, PR>
where
    A: TransactionPool<Block = Block> + 'static,
    B: backend::Backend<Block> + Send + Sync + 'static,
    Block: BlockT,
    C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>> + BlockBuilderApi<Block>,
    PR: ProofRecording,
{
    type Transaction = backend::TransactionFor<B, Block>;
    type Proposal =
        Pin<Box<dyn Future<Output = Result<Proposal<Block, Self::Transaction, PR::Proof>, Self::Error>> + Send>>;
    type Error = sp_blockchain::Error;
    type ProofRecording = PR;
    type Proof = PR::Proof;

    fn propose(
        self,
        inherent_data: InherentData,
        inherent_digests: Digest,
        max_duration: time::Duration,
        block_size_limit: Option<usize>,
    ) -> Self::Proposal {
        let (tx, rx) = oneshot::channel();
        let spawn_handle = self.spawn_handle.clone();

        spawn_handle.spawn_blocking(
            "madara-block-proposer",
            None,
            Box::pin(async move {
                // Leave some time for evaluation and block finalization (20%)
                // and some time for block production (80%).
                // We need to benchmark and tune this value.
                // Open question: should we make this configurable?
                let deadline = (self.now)() + max_duration - max_duration / 5;
                let res = self.propose_with(inherent_data, inherent_digests, deadline, block_size_limit).await;
                if tx.send(res).is_err() {
                    trace!("Could not send block production result to proposer!");
                }
            }),
        );

        async move { rx.await? }.boxed()
    }
}

/// If the block is full we will attempt to push at most
/// this number of transactions before quitting for real.
/// It allows us to increase block utilization.
const MAX_SKIPPED_TRANSACTIONS: usize = 8;

impl<A, B, Block, C, PR> Proposer<B, Block, C, A, PR>
where
    A: TransactionPool<Block = Block>,
    B: backend::Backend<Block> + Send + Sync + 'static,
    Block: BlockT,
    C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>> + BlockBuilderApi<Block>,
    PR: ProofRecording,
{
    /// Propose a new block.
    ///
    /// # Arguments
    /// * `inherents` - The inherents to include in the block.
    /// * `inherent_digests` - The inherent digests to include in the block.
    /// * `deadline` - The deadline for proposing the block.
    /// * `block_size_limit` - The maximum size of the block in bytes.
    ///
    ///
    /// The function follows these general steps:
    /// 1. Starts a timer to measure the total time it takes to create the proposal.
    /// 2. Initializes a new block at the parent hash with the given inherent digests.
    /// 3. Iterates over the inherents and pushes them into the block builder. Handles any potential
    /// errors.
    /// 4. Sets up the soft deadline and starts the block timer.
    /// 5. Gets an iterator over the pending transactions and iterates over them.
    /// 6. Checks the deadline and handles the case when the deadline is reached.
    /// 7. Checks the block size limit and handles cases where transactions would cause the block to
    /// exceed the limit.
    /// 8. Attempts to push the transaction into the block and handles any
    /// potential errors.
    /// 9. If the block size limit was reached without adding any transaction,
    /// it logs a warning.
    /// 10. Removes invalid transactions from the pool.
    /// 11. Builds the block and updates the metrics.
    /// 12. Converts the storage proof to the required format.
    /// 13. Measures the total time it took to create the proposal and updates the corresponding
    /// metric.
    /// 14. Returns a new `Proposal` with the block, proof, and storage changes.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The block cannot be created at the parent hash.
    /// - Any of the inherents cannot be pushed into the block builder.
    /// - The block cannot be built.
    /// - The storage proof cannot be converted into the required format.
    async fn propose_with(
        self,
        inherent_data: InherentData,
        inherent_digests: Digest,
        deadline: time::Instant,
        block_size_limit: Option<usize>,
    ) -> Result<Proposal<Block, backend::TransactionFor<B, Block>, PR::Proof>, sp_blockchain::Error> {
        // Start the timer to measure the total time it takes to create the proposal.
        let propose_with_timer = time::Instant::now();

        // Initialize a new block builder at the parent hash with the given inherent digests.
        let mut block_builder = self.client.new_block_at(self.parent_hash, inherent_digests, PR::ENABLED)?;

        self.apply_inherents(&mut block_builder, inherent_data)?;

        let block_timer = time::Instant::now();

        // Apply transactions and record the reason why we stopped.
        let end_reason = self.apply_extrinsics(&mut block_builder, deadline, block_size_limit).await?;

        // Build the block.
        let (block, storage_changes, proof) = block_builder.build()?.into_inner();

        // Measure the total time it took to build the block.
        let block_took = block_timer.elapsed();

        // Convert the storage proof into the required format.
        let proof = PR::into_proof(proof).map_err(|e| sp_blockchain::Error::Application(Box::new(e)))?;

        // Print the summary of the proposal.
        self.print_summary(&block, end_reason, block_took, propose_with_timer.elapsed());
        Ok(Proposal { block, proof, storage_changes })
    }

    /// Apply all inherents to the block.
    /// This function will return an error if any of the inherents cannot be pushed into the block
    /// builder. It will also update the metrics.
    /// # Arguments
    /// * `block_builder` - The block builder to push the inherents into.
    /// * `inherent_data` - The inherents to push into the block builder.
    /// # Returns
    /// This function will return `Ok(())` if all inherents were pushed into the block builder.
    /// # Errors
    /// This function will return an error if any of the inherents cannot be pushed into the block
    /// builder.
    fn apply_inherents(
        &self,
        block_builder: &mut sc_block_builder::BlockBuilder<'_, Block, C, B>,
        inherent_data: InherentData,
    ) -> Result<(), sp_blockchain::Error> {
        let create_inherents_start = time::Instant::now();
        let inherents = block_builder.create_inherents(inherent_data)?;
        let create_inherents_end = time::Instant::now();

        self.metrics.report(|metrics| {
            metrics
                .create_inherents_time
                .observe(create_inherents_end.saturating_duration_since(create_inherents_start).as_secs_f64());
        });

        for inherent in inherents {
            match block_builder.push(inherent) {
                Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
                    warn!(target: LOG_TARGET, "⚠️  Dropping non-mandatory inherent from overweight block.")
                }
                Err(ApplyExtrinsicFailed(Validity(e))) if e.was_mandatory() => {
                    error!("❌️ Mandatory inherent extrinsic returned error. Block cannot be produced.");
                    return Err(ApplyExtrinsicFailed(Validity(e)));
                }
                Err(e) => {
                    warn!(target: LOG_TARGET, "❗️ Inherent extrinsic returned unexpected error: {}. Dropping.", e);
                }
                Ok(_) => {}
            }
        }
        Ok(())
    }

    /// Apply as many extrinsics as possible to the block.
    /// This function will return an error if the block cannot be built.
    /// # Arguments
    /// * `block_builder` - The block builder to push the extrinsics into.
    /// * `deadline` - The deadline to stop applying extrinsics.
    /// * `block_size_limit` - The maximum size of the block.
    /// # Returns
    /// The reason why we stopped applying extrinsics.
    /// # Errors
    /// This function will return an error if the block cannot be built.
    async fn apply_extrinsics(
        &self,
        block_builder: &mut sc_block_builder::BlockBuilder<'_, Block, C, B>,
        deadline: time::Instant,
        block_size_limit: Option<usize>,
    ) -> Result<EndProposingReason, sp_blockchain::Error> {
        // proceed with transactions
        // We calculate soft deadline used only in case we start skipping transactions.
        let now = (self.now)();
        let left = deadline.saturating_duration_since(now);
        let left_micros: u64 = left.as_micros().saturated_into();
        let soft_deadline = now + time::Duration::from_micros(self.soft_deadline_percent.mul_floor(left_micros));
        let mut skipped = 0;
        let mut unqueue_invalid = Vec::new();

        let mut t1 = self.transaction_pool.ready_at(self.parent_number).fuse();
        let mut t2 = futures_timer::Delay::new(deadline.saturating_duration_since((self.now)()) / 8).fuse();

        let mut pending_iterator = select! {
            res = t1 => res,
            _ = t2 => {
                warn!(target: LOG_TARGET,
                    "Timeout fired waiting for transaction pool at block #{}. \
                    Proceeding with production.",
                    self.parent_number,
                );
                self.transaction_pool.ready()
            },
        };

        let block_size_limit = block_size_limit.unwrap_or(self.default_block_size_limit);

        debug!(target: LOG_TARGET, "Attempting to push transactions from the pool.");
        debug!(target: LOG_TARGET, "Pool status: {:?}", self.transaction_pool.status());
        let mut transaction_pushed = false;

        let end_reason = loop {
            let pending_tx = if let Some(pending_tx) = pending_iterator.next() {
                pending_tx
            } else {
                break EndProposingReason::NoMoreTransactions;
            };

            let now = (self.now)();
            if now > deadline {
                debug!(
                    target: LOG_TARGET,
                    "Consensus deadline reached when pushing block transactions, proceeding with proposing."
                );
                break EndProposingReason::HitDeadline;
            }

            let pending_tx_data = pending_tx.data().clone();
            let pending_tx_hash = pending_tx.hash().clone();

            let block_size = block_builder.estimate_block_size(false);
            if block_size + pending_tx_data.encoded_size() > block_size_limit {
                pending_iterator.report_invalid(&pending_tx);
                if skipped < MAX_SKIPPED_TRANSACTIONS {
                    skipped += 1;
                    debug!(
                        target: LOG_TARGET,
                        "Transaction would overflow the block size limit, but will try {} more transactions before \
                         quitting.",
                        MAX_SKIPPED_TRANSACTIONS - skipped,
                    );
                    continue;
                } else if now < soft_deadline {
                    debug!(
                        target: LOG_TARGET,
                        "Transaction would overflow the block size limit, but we still have time before the soft \
                         deadline, so we will try a bit more."
                    );
                    continue;
                } else {
                    debug!(target: LOG_TARGET, "Reached block size limit, proceeding with proposing.");
                    break EndProposingReason::HitBlockSizeLimit;
                }
            }

            trace!(target: LOG_TARGET, "[{:?}] Pushing to the block.", pending_tx_hash);
            match sc_block_builder::BlockBuilder::push(block_builder, pending_tx_data) {
                Ok(()) => {
                    transaction_pushed = true;
                    debug!(target: LOG_TARGET, "[{:?}] Pushed to the block.", pending_tx_hash);
                }
                Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
                    pending_iterator.report_invalid(&pending_tx);
                    if skipped < MAX_SKIPPED_TRANSACTIONS {
                        skipped += 1;
                        debug!(
                            target: LOG_TARGET,
                            "Block seems full, but will try {} more transactions before quitting.",
                            MAX_SKIPPED_TRANSACTIONS - skipped,
                        );
                    } else if (self.now)() < soft_deadline {
                        debug!(
                            target: LOG_TARGET,
                            "Block seems full, but we still have time before the soft deadline, so we will try a bit \
                             more before quitting."
                        );
                    } else {
                        debug!(target: LOG_TARGET, "Reached block weight limit, proceeding with proposing.");
                        break EndProposingReason::HitBlockWeightLimit;
                    }
                }
                Err(e) => {
                    pending_iterator.report_invalid(&pending_tx);
                    debug!(target: LOG_TARGET, "[{:?}] Invalid transaction: {}", pending_tx_hash, e);
                    unqueue_invalid.push(pending_tx_hash);
                }
            }
        };

        if matches!(end_reason, EndProposingReason::HitBlockSizeLimit) && !transaction_pushed {
            warn!(
                target: LOG_TARGET,
                "Hit block size limit of `{}` without including any transaction!", block_size_limit,
            );
        }

        self.transaction_pool.remove_invalid(&unqueue_invalid);
        Ok(end_reason)
    }

    /// Prints a summary and does telemetry + metrics.
    /// This is called after the block is created.
    /// # Arguments
    /// * `block` - The block that was created.
    /// * `end_reason` - The reason why we stopped adding transactions to the block.
    /// * `block_took` - The time it took to create the block.
    /// * `propose_with_took` - The time it took to propose the block.
    fn print_summary(
        &self,
        block: &Block,
        end_reason: EndProposingReason,
        block_took: time::Duration,
        propose_with_took: time::Duration,
    ) {
        let extrinsics = block.extrinsics();
        self.metrics.report(|metrics| {
            metrics.number_of_transactions.set(extrinsics.len() as u64);
            metrics.block_constructed.observe(block_took.as_secs_f64());
            metrics.report_end_proposing_reason(end_reason);
            metrics.create_block_proposal_time.observe(propose_with_took.as_secs_f64());
        });

        let extrinsics_summary = if extrinsics.is_empty() {
            "no extrinsics".to_string()
        } else {
            format!("extrinsics ({})", extrinsics.len(),)
        };

        info!(
            "🥷 Prepared block for proposing at {} ({} ms) [hash: {:?}; parent_hash: {}; {extrinsics_summary}",
            block.header().number(),
            block_took.as_millis(),
            block.header().hash(),
            block.header().parent_hash(),
        );
    }
}
