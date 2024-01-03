pub mod avail;
pub mod celestia;
pub mod ethereum;
mod sharp;
pub mod utils;

use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::types::{I256, U256};
use futures::channel::mpsc;
use futures::StreamExt;
use mc_commitment_state_diff::BlockDAData;
use mp_hashers::HasherT;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block as BlockT;
use starknet_api::block::BlockHash;
use starknet_api::state::ThinStateDiff;
use utils::state_diff_to_calldata;

pub struct DataAvailabilityWorker<B, H>(PhantomData<(B, H)>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum DaLayer {
    Celestia,
    Ethereum,
    Avail,
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

#[async_trait]
pub trait DaClient: Send + Sync {
    fn get_mode(&self) -> DaMode;
    async fn last_published_state(&self) -> Result<I256>;
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()>;
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
        mut state_diffs_rx: mpsc::Receiver<BlockDAData>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) {
        while let Some(BlockDAData(starknet_block_hash, csd, num_addr_accessed)) = state_diffs_rx.next().await {
            log::info!(
                "received state diff for block {starknet_block_hash}: {csd:?}.{num_addr_accessed} addresses accessed."
            );

            let da_client = da_client.clone();
            let madara_backend = madara_backend.clone();
            tokio::spawn(async move {
                match prove(da_client.get_mode(), starknet_block_hash, &csd, num_addr_accessed, madara_backend.clone())
                    .await
                {
                    Err(err) => log::error!("proving error: {err}"),
                    Ok(()) => {}
                }

                match update_state::<B, H>(madara_backend, da_client, starknet_block_hash, csd, num_addr_accessed).await
                {
                    Err(err) => log::error!("state publishing error: {err}"),
                    Ok(()) => {}
                };
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
    log::info!("proving the block {block_hash}");

    match da_mode {
        DaMode::Validity => {
            // Submit the Starknet OS PIE
            // TODO: Validity Impl
            // run the Starknet OS with the Cairo VM
            // extract the PIE from the Cairo VM run
            // pass the PIE to `submit_pie` and zip/base64 internal
            if let Ok(job_resp) = sharp::submit_pie("TODO") {
                log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                // Store the cairo job key
                madara_backend
                    .da()
                    .update_cairo_job(&block_hash, job_resp.cairo_job_key)
                    .map_err(|e| anyhow!("{e}"))?;
            }
        }
        _ => {
            log::info!("don't prove in remaining DA modes")
        }
    }

    Ok(())
}

pub async fn update_state<B: BlockT, H: HasherT>(
    madara_backend: Arc<mc_db::Backend<B>>,
    da_client: Arc<dyn DaClient + Send + Sync>,
    starknet_block_hash: BlockHash,
    csd: ThinStateDiff,
    num_addr_accessed: usize,
) -> Result<(), anyhow::Error> {
    // store the state diff
    madara_backend
        .da()
        .store_state_diff(&starknet_block_hash, state_diff_to_calldata(csd, num_addr_accessed))
        .map_err(|e| anyhow!("{e}"))?;

    // Query last written state
    // TODO: this value will be used to ensure the correct state diff is being written in
    // Validity mode
    let _last_published_state = da_client.last_published_state().await?;

    match da_client.get_mode() {
        DaMode::Validity => {
            // Check the SHARP status of last_proved + 1
            // Write the publish state diff of last_proved + 1
            log::info!("validity da mode not implemented");
        }
        DaMode::Sovereign => match madara_backend.da().state_diff(&starknet_block_hash) {
            Ok(state_diff) => {
                da_client.publish_state_diff(state_diff).await.map_err(|e| anyhow!("DA PUBLISH ERROR: {e}"))?;
            }
            Err(e) => Err(anyhow!("could not pull state diff for block {starknet_block_hash}: {e}"))?,
        },
        DaMode::Volition => log::info!("volition da mode not implemented"),
    };

    Ok(())
}
