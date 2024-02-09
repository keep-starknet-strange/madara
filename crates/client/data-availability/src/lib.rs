#[cfg(feature = "avail")]
pub mod avail;
#[cfg(feature = "celestia")]
pub mod celestia;
pub mod ethereum;
mod sharp;
pub mod utils;

mod da_metrics;

use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::types::{I256, U256};
use futures::channel::mpsc;
use futures::StreamExt;
use mc_commitment_state_diff::BlockDAData;
use mp_hashers::HasherT;
use prometheus_endpoint::prometheus::core::AtomicU64;
use prometheus_endpoint::{register, Gauge, Opts, Registry as PrometheusRegistry};
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block as BlockT;
use starknet_api::block::BlockHash;
use starknet_api::state::ThinStateDiff;
use thiserror::Error;
use utils::block_data_to_calldata;

use crate::da_metrics::DaMetrics;

pub struct DataAvailabilityWorker<B, H>(PhantomData<(B, H)>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum DaLayer {
    #[cfg(feature = "celestia")]
    Celestia,
    Ethereum,
    #[cfg(feature = "avail")]
    Avail,
}

#[derive(Error, Debug)]
pub enum DaError {
    #[error("failed opening config: {0}")]
    FailedOpeningConfig(std::io::Error),
    #[error("failed parsing config: {0}")]
    FailedParsingConfig(serde_json::Error),
    #[error("failed converting parameter: {0}")]
    FailedConversion(anyhow::Error),
    #[error("failed building client: {0}")]
    FailedBuildingClient(anyhow::Error),
    #[error("failed submitting data through client: {0}")]
    FailedDataSubmission(anyhow::Error),
    #[error("failed fetching data through client: {0}")]
    FailedDataFetching(anyhow::Error),
    #[error("failed validating data: {0}")]
    FailedDataValidation(anyhow::Error),
    #[error("Invalid http endpoint: {0}")]
    InvalidHttpEndpoint(String),
}

impl Display for DaLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "celestia")]
            DaLayer::Celestia => Display::fmt("Celestia", f),
            DaLayer::Ethereum => Display::fmt("Ethereum", f),
            #[cfg(feature = "avail")]
            DaLayer::Avail => Display::fmt("Avail", f),
        }
    }
}

/// Data availability modes in which Madara can be initialized.
///
/// Default only mode currently implemented is Sovereing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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

impl Display for DaMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaMode::Validity => Display::fmt("Validity", f),
            DaMode::Volition => Display::fmt("Volition", f),
            DaMode::Sovereign => Display::fmt("Sovereign", f),
        }
    }
}

#[async_trait]
pub trait DaClient: Send + Sync {
    fn get_mode(&self) -> DaMode;
    async fn last_published_state(&self) -> Result<I256>;
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()>;
    fn get_da_metric_labels(&self) -> HashMap<String, String>;
}

/// The client worker for DA related tasks
///
/// Listen to new block state diff and spawn new threads to execute each block flow concurently.
/// The flow goes as follow:
/// 1. Prove. Do nothing if node is run in sovereign mode
/// 2. Updata
impl<B, H> DataAvailabilityWorker<B, H>
where
    B: BlockT,
    H: HasherT,
{
    pub async fn prove_current_block(
        da_client: Arc<dyn DaClient + Send + Sync>,
        prometheus: Option<PrometheusRegistry>,
        mut state_diffs_rx: mpsc::Receiver<BlockDAData>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) {
        let da_metrics = prometheus.as_ref().and_then(|registry| DaMetrics::register(registry).ok());
        if let Some(registry) = prometheus.as_ref() {
            let gauge = Gauge::<AtomicU64>::with_opts(
                Opts::new("madara_da_layer_info", "Information about the data availability layer used")
                    .const_labels(da_client.get_da_metric_labels()),
            );
            match gauge {
                Ok(gauge) => match register(gauge, registry) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("failed to register gauge for da layer info metrics: {e}");
                    }
                },
                Err(e) => {
                    log::error!("failed to create gauge for da layer info metrics: {e}");
                }
            }
        }
        while let Some(block_da_data) = state_diffs_rx.next().await {
            log::info!("Received state diff for block {}", block_da_data.block_hash);

            let da_metrics = da_metrics.clone();
            let da_client = da_client.clone();
            let madara_backend = madara_backend.clone();
            tokio::spawn(async move {
                let prove_state_start = time::Instant::now();

                if let Err(err) = prove(
                    da_client.get_mode(),
                    block_da_data.block_hash,
                    &block_da_data.state_diff,
                    block_da_data.num_addr_accessed,
                    madara_backend.clone(),
                )
                .await
                {
                    log::error!("Failed to prove block: {err}");
                }
                let prove_state_end = time::Instant::now();

                if let Err(err) = update_state::<B, H>(madara_backend, da_client, block_da_data).await {
                    log::error!("Failed to update the DA state: {err}");
                };
                let update_state_end = time::Instant::now();

                if let Some(da_metrics) = da_metrics {
                    da_metrics
                        .state_proofs
                        .observe(prove_state_end.saturating_duration_since(prove_state_start).as_secs_f64());
                    da_metrics
                        .state_updates
                        .observe(update_state_end.saturating_duration_since(prove_state_end).as_secs_f64());
                }
            });
        }
    }
}

pub async fn prove<B: BlockT>(
    da_mode: DaMode,
    block_hash: BlockHash,
    _state_diff: &ThinStateDiff,
    _num_addr_accessed: usize,
    madara_backend: Arc<mc_db::Backend<B>>,
) -> Result<(), anyhow::Error> {
    match da_mode {
        DaMode::Validity => {
            // Submit the Starknet OS PIE
            // TODO: Validity Impl
            // run the Starknet OS with the Cairo VM
            // extract the PIE from the Cairo VM run
            // pass the PIE to `submit_pie` and zip/base64 internal
            if let Ok(job_resp) = sharp::submit_pie("TODO") {
                log::info!("Proof job submitted with key '{}'", job_resp.cairo_job_key);
                // Store the cairo job key
                madara_backend
                    .da()
                    .update_cairo_job(&block_hash, job_resp.cairo_job_key)
                    .map_err(|e| anyhow!("{e}"))?;
            }
        }
        _ => {
            log::info!("No proof required for current DA mode ({da_mode}).")
        }
    }

    Ok(())
}

pub async fn update_state<B: BlockT, H: HasherT>(
    madara_backend: Arc<mc_db::Backend<B>>,
    da_client: Arc<dyn DaClient + Send + Sync>,
    block_da_data: BlockDAData,
) -> Result<(), anyhow::Error> {
    let block_hash = block_da_data.block_hash;

    // store the state diff
    madara_backend.da().store_state_diff(&block_hash, &block_da_data.state_diff).map_err(|e| anyhow!("{e}"))?;

    // Query last written state
    // TODO: this value will be used to ensure the correct state diff is being written in
    // Validity mode
    let _last_published_state = da_client.last_published_state().await?;

    match da_client.get_mode() {
        DaMode::Validity => {
            // Check the SHARP status of last_proved + 1
            // Write the publish state diff of last_proved + 1
            log::info!("[VALIDITY] not implemented");
        }
        DaMode::Sovereign => {
            let calldata = block_data_to_calldata(block_da_data);
            da_client.publish_state_diff(calldata).await.map_err(|e| anyhow!("[SOVEREIGN] publish error: {e}"))?
        }
        DaMode::Volition => log::info!("[VOLITION] not implemented"),
    };

    Ok(())
}
