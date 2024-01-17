use std::sync::Arc;

use futures::StreamExt;
use futures_timer::Delay;
use mp_block::Block as StarknetBlock;
use mp_hashers::HasherT;
use mp_messages::{MessageL1ToL2, MessageL2ToL1};
use mp_snos_output::StarknetOsOutput;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::Transaction;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_client_api::BlockchainEvents;
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::hash::StarkHash;
use starknet_api::transaction::TransactionHash;

use crate::errors::Error;
use crate::{Result, RetryStrategy, SettlementProvider, SettlementWorker, StarknetSpec, StarknetState};

impl<B, H, SC> SettlementWorker<B, H, SC>
where
    B: BlockT,
    H: HasherT,
    SC: ProvideRuntimeApi<B> + HeaderBackend<B> + BlockchainEvents<B>,
    SC::Api: StarknetRuntimeApi<B>,
{
    /// A thread responsible for updating (progressing) Starknet state on the settlement layer.
    /// For now we use a simplified setup without validity proofs & DA, but in the future
    /// Starknet state contract will also validate state transition against the fact registry.
    /// That means we will need to publish state diffs and STARK proof before state update.
    ///
    /// This is an external loop that is responsible for handling temporary (recoverable) errors.
    pub async fn sync_state(
        substrate_client: Arc<SC>,
        settlement_provider: Box<dyn SettlementProvider<B>>,
        madara_backend: Arc<mc_db::Backend<B>>,
        retry_strategy: Box<dyn RetryStrategy<B>>,
    ) {
        loop {
            match Self::sync_state_loop(&substrate_client, settlement_provider.as_ref(), &madara_backend).await {
                Ok(()) => {
                    return;
                }
                Err(err) => {
                    log::error!("[settlement] {err}");
                    match retry_strategy.can_retry(&err) {
                        Some(dur) => {
                            log::info!("[settlement] Retrying after {} ms", dur.as_millis());
                            Delay::new(dur).await;
                            continue;
                        }
                        None => panic!("Unrecoverable error in settlement thread: {}", err),
                    }
                }
            }
        }
    }

    /// This is an internal loop that listens to the new finalized blocks
    /// and attempts to settle the state.
    ///
    /// It works as follows:
    ///
    /// 1. First of all it retrieves the latest settled state
    /// 2. Then it starts to listen for new finality notifications
    /// 3. For all incoming blocks with height lower than the settled one it checks the state root
    ///    validity.
    /// Inconsistent state root means we have a fatal error, which cannot be resolved automatically.
    ///
    /// 4. Once it gets up to the tip of the chain it starts to apply new state updates.
    /// It is possible that there is a need to apply multiple state updates.
    ///
    /// Sync state loop operates as long as there are new blocks being finalized.
    /// In case chain is stuck it won't update the state, even if there are pending blocks.
    /// It is ok, since it's not a normal condition, and generally we expect that the chain will
    /// advance indefinitely.
    async fn sync_state_loop<SP>(
        substrate_client: &SC,
        settlement_provider: &SP,
        madara_backend: &mc_db::Backend<B>,
    ) -> Result<(), B>
    where
        SP: ?Sized + SettlementProvider<B>,
    {
        if !settlement_provider.is_initialized().await? {
            return Err(Error::StateNotInitialized);
        }

        let starknet_spec = settlement_provider.get_chain_spec().await?;
        log::debug!("[settlement] Starknet chain spec {:?}", starknet_spec);

        // We need to make sure that we are on the same page with the settlement contract.
        Self::verify_starknet_spec(substrate_client, &starknet_spec)?;

        let mut last_settled_state = settlement_provider.get_state().await?;
        log::debug!("[settlement] Last settled state {:?}", last_settled_state);

        // If we haven't reached the settled level yet (e.g. syncing from scratch) this check will pass.
        // But we need to run it again once we are up to speed.
        Self::verify_starknet_state(substrate_client, &last_settled_state, madara_backend)?;

        let mut finality_notifications = substrate_client.finality_notification_stream();
        let mut sync_from: u64 = last_settled_state.block_number.try_into()?;

        while let Some(notification) = finality_notifications.next().await {
            let block = mp_digest_log::find_starknet_block(notification.header.digest())?;
            let sync_to = block.header().block_number;

            if sync_from > sync_to {
                log::info!("[settlement] Skipping block {} (already settled)", sync_to);
                continue;
            }

            if sync_from == sync_to {
                log::info!("[settlement] Verifying state root for block {}", sync_to);
                Self::verify_starknet_state(substrate_client, &last_settled_state, madara_backend)?;
                continue;
            }

            log::info!("[settlement] Syncing state for blocks {} -> {}", sync_from, sync_to);
            while sync_from < sync_to {
                let (next_block, substrate_block_hash) = if sync_from + 1 == sync_to {
                    // This is a typical scenario when we are up to speed with the chain
                    (block.clone(), notification.hash)
                } else {
                    Self::get_starknet_block(substrate_client, sync_from + 1)?
                };

                let new_state = Self::update_starknet_state(
                    substrate_client,
                    settlement_provider,
                    &last_settled_state,
                    &next_block,
                    substrate_block_hash,
                    starknet_spec.config_hash,
                    madara_backend,
                )
                .await?;

                log::debug!("[settlement] State transitioned to {:?}", new_state);
                last_settled_state = new_state;
                sync_from += 1;
            }
        }

        Ok(())
    }

    /// Returns Starknet block given it's height (level, number).
    /// The trick here is that Starknet blocks are embedded into Substrate blocks.
    ///
    /// Firstly, we need to get Substrate block hash by the Starknet block height.
    /// This mapping is kept in a separate storage and there is a dedicated thread which maintains
    /// it. There might be situations when we cannot resolve the query (e.g. our node is out of
    /// sync), but eventually it will be ok.
    ///
    /// Secondly, we try to find a corresponding Substrate block (header) by its hash.
    /// Lastly, we extract Starknet block from the Substrate block digest.
    fn get_starknet_block(substrate_client: &SC, block_number: u64) -> Result<(StarknetBlock, B::Hash), B> {
        let substrate_block_hash = substrate_client
            .hash(UniqueSaturatedInto::unique_saturated_into(block_number))?
            .ok_or_else(|| Error::UnknownStarknetBlock(block_number))?;

        let substrate_block_header = substrate_client
            .header(substrate_block_hash)?
            .ok_or_else(|| Error::UnknownSubstrateBlock(substrate_block_hash))?;

        let starknet_block = mp_digest_log::find_starknet_block(substrate_block_header.digest())?;

        Ok((starknet_block, substrate_block_hash))
    }

    /// Checks that settlement contract is initialized with the same program & config hash as
    /// Madara.
    fn verify_starknet_spec(substrate_client: &SC, starknet_spec: &StarknetSpec) -> Result<(), B> {
        let substrate_hash = substrate_client.info().best_hash;
        let program_hash: StarkHash = substrate_client.runtime_api().program_hash(substrate_hash)?.into();

        if starknet_spec.program_hash != program_hash {
            return Err(Error::ProgramHashMismatch { expected: program_hash, actual: starknet_spec.program_hash });
        }

        let config_hash = substrate_client.runtime_api().config_hash(substrate_hash)?;

        if starknet_spec.config_hash != config_hash {
            return Err(Error::ConfigHashMismatch { expected: config_hash, actual: starknet_spec.config_hash });
        }

        Ok(())
    }

    /// Tries to verify that the state root for the specified block in Madara storage
    /// is the same as in the given state.
    ///
    /// If Madara chain haven't reached the given block level yet, it returns OK, assuming that as
    /// soon as it catches up - this check will be done again.
    fn verify_starknet_state(
        substrate_client: &SC,
        state: &StarknetState,
        madara_backend: &mc_db::Backend<B>,
    ) -> Result<(), B> {
        let height: u64 = state.block_number.try_into()?;

        match Self::get_starknet_block(substrate_client, height) {
            Ok((_block, _)) => {
                let state_root = madara_backend.temporary_global_state_root_getter();
                // Verify that current onchain state is consistent with corresponding Madara block
                if state.state_root != state_root {
                    return Err(Error::StateRootMismatch { height, expected: state_root, actual: state.state_root });
                }
                Ok(())
            }
            Err(Error::UnknownStarknetBlock(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Aggregates Starknet OS output from a given Starknet block and tries to settle it using a
    /// particular provider.
    ///
    /// "Main part" of Starknet OS program output consists of:
    ///  - previous state root (at the beginning of the block)
    ///  - next state root (at the end of the block)
    ///  - block number
    ///  - config hash
    ///  - list of messages transferred between L1 and L2
    ///
    /// We construct it using fast execution results, without producing the execution trace which is
    /// used for STARK proof. Still it must match the output got from the respective circuit,
    /// otherwise the settlement will fail.
    async fn update_starknet_state<SP>(
        substrate_client: &SC,
        settlement_provider: &SP,
        prev_state: &StarknetState,
        next_block: &StarknetBlock,
        substrate_block_hash: B::Hash,
        config_hash: StarkHash,
        madara_backend: &mc_db::Backend<B>,
    ) -> Result<StarknetState, B>
    where
        SP: ?Sized + SettlementProvider<B>,
    {
        let next_state = StarknetState {
            block_number: next_block.header().block_number.into(),
            state_root: madara_backend.temporary_global_state_root_getter(),
        };

        let mut messages_to_l1: Vec<MessageL2ToL1> = Vec::new();
        let mut messages_to_l2: Vec<MessageL1ToL2> = Vec::new();

        let chain_id = substrate_client.runtime_api().chain_id(substrate_block_hash)?;

        for tx in next_block.transactions() {
            if let Transaction::L1Handler(l1_handler) = tx {
                messages_to_l2.push(l1_handler.clone().into());
            }
            let tx_hash = TransactionHash(tx.compute_hash::<H>(chain_id, false).into());
            substrate_client
                .runtime_api()
                .get_tx_messages_to_l1(substrate_block_hash, tx_hash)?
                .into_iter()
                .for_each(|msg| messages_to_l1.push(msg.into()));
        }

        // See https://github.com/starkware-libs/cairo-lang/blob/27a157d761ae49b242026bcbe5fca6e60c1e98bd/src/starkware/starknet/core/os/output.cairo
        let program_output = StarknetOsOutput {
            prev_state_root: prev_state.state_root,
            new_state_root: next_state.state_root,
            block_number: next_state.block_number,
            block_hash: next_block.header().hash::<H>().into(),
            config_hash,
            messages_to_l1,
            messages_to_l2,
        };
        log::trace!("{:#?}", program_output);

        settlement_provider.update_state(program_output).await?;

        Ok(next_state)
    }
}
