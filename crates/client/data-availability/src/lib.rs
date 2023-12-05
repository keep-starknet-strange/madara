pub mod avail;
pub mod celestia;
pub mod ethereum;
mod sharp;
pub mod utils;

use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use blockifier::state::cached_state::CommitmentStateDiff;
use ethers::types::{I256, U256};
use futures::channel::mpsc;
use futures::StreamExt;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_client_api::client::BlockchainEvents;
use serde::Deserialize;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::block::BlockHash;
use utils::{safe_split, state_diff_to_calldata};

pub struct DataAvailabilityWorker<B, C>(PhantomData<(B, C)>);

#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum DaLayer {
    Celestia,
    Ethereum,
    Avail,
}

/// Data availability modes in which Madara can be initialized.
///
/// Default only mode currently implemented is Sovereign.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Default)]
pub enum DaMode {
    /// Full Validity Rollup
    ///
    /// Generates a Cairo execution trace of the StarknetOS
    /// run for the given block as it is applied to the current Madara state.
    /// Once this execution trace is proved to the L1 Verifier(i.e. [Ethereum](https://goerli.etherscan.io/address/0x8f97970aC5a9aa8D130d35146F5b59c4aef57963))
    /// the relevant [state diff](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/on-chain-data) can be written and validated against the on-chain
    /// proof verification of the block propagation.
    #[serde(rename = "validity")]
    Validity,
    /// Hybrid Volition
    ///
    /// Volitions allow applications and users to interoperate between on-chain data and off-chain
    /// da. Although full specs are not currently available, this mode will entail generating
    /// a StarknetOS execution trace for data elected to be on-chain and interaction w/ the prover
    /// will be necessary.
    #[serde(rename = "volition")]
    Volition,
    /// Sovereign Rollup
    ///
    /// Sovereign state diffs are untethered to an accompanying validity proof therefore
    /// they can simply be published to any da solution available. As this solution does not
    /// require an execution trace to be proved we can simply parse the state diff from the
    /// storage changes of the block.
    #[serde(rename = "sovereign")]
    #[default]
    Sovereign,
}

#[async_trait]
pub trait DaClient: Send + Sync {
    fn get_mode(&self) -> DaMode;
    async fn last_published_state(&self) -> Result<I256>;
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()>;
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + 'static,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn prove_current_block(
        da_mode: DaMode,
        mut state_diffs_rx: mpsc::Receiver<(BlockHash, CommitmentStateDiff)>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) {
        while let Some((block_hash, csd)) = state_diffs_rx.next().await {
            // store the da encoded calldata for the state update worker
            if let Err(db_err) = madara_backend
                .da()
                .store_state_diff(&block_hash, state_diff_to_calldata(csd, csd.address_to_class_hash.len()))
            {
                log::error!("db err: {db_err}");
            };

            match da_mode {
                DaMode::Validity => {
                    // TODO:
                    // - run the StarknetOs for this block
                    // - parse the PIE to `submit_pie` and zip/base64 internal
                    if let Ok(job_resp) = sharp::submit_pie("TODO") {
                        log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                        // Store the cairo job key
                        if let Err(db_err) = madara_backend.da().update_cairo_job(&block_hash, job_resp.cairo_job_key) {
                            log::error!("db err: {db_err}");
                        };
                    }
                }
                _ => {
                    log::info!("don't prove in remaining DA modes")
                }
            }
        }
    }
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn update_state(
        da_client: Box<dyn DaClient + Send + Sync>,
        client: Arc<C>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last written state
            // TODO: this value will be used to ensure the correct state diff is being written in Validity mode
            let _last_published_state = match da_client.last_published_state().await {
                Ok(last_published_state) => last_published_state,
                Err(e) => {
                    log::error!("da provider error: {e}");
                    continue;
                }
            };

            match da_client.get_mode() {
                DaMode::Validity => {
                    // Check the SHARP status of last_proved + 1
                    // Write the publish state diff of last_proved + 1
                    log::info!("validity da mode not implemented");
                }
                DaMode::Sovereign => match madara_backend.da().state_diff(&notification.hash) {
                    Ok(state_diff) => {
                        if let Err(e) = da_client.publish_state_diff(state_diff).await {
                            log::error!("DA PUBLISH ERROR: {}", e);
                        }
                    }
                    Err(e) => log::error!("could not pull state diff: {e}"),
                },
                DaMode::Volition => log::info!("volition da mode not implemented"),
            }
        }
    }
}
