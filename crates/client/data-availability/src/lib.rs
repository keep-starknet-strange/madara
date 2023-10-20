pub mod avail;
pub mod celestia;
pub mod ethereum;
mod sharp;
pub mod utils;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ethers::types::{I256, U256};
use futures::StreamExt;
use mp_storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_NONCE, STARKNET_STORAGE,
};
use sc_client_api::client::BlockchainEvents;
use sc_client_api::StorageData;
use serde::Deserialize;
use sp_api::ProvideRuntimeApi;
use sp_io::hashing::twox_128;
use sp_runtime::traits::Block as BlockT;
use starknet_api::api_core::ContractAddress;
use starknet_core::types::StateDiff;

pub type StorageWrites<'a> = Vec<(&'a [u8], &'a [u8])>;

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
    /// proof verification of the block propogation.
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

fn safe_split(key: &[u8]) -> ([u8; 16], [u8; 16], Option<Vec<u8>>) {
    let length = key.len();
    let (mut prefix, mut child, mut rest) = ([0_u8; 16], [0_u8; 16], None);
    if length <= 16 {
        prefix[..length].copy_from_slice(&key[..])
    }
    if length > 16 && key.len() <= 32 {
        prefix.copy_from_slice(&key[..16]);
        child[..(length - 16)].copy_from_slice(&key[16..]);
    }
    if length > 32 {
        prefix.copy_from_slice(&key[..16]);
        child.copy_from_slice(&key[16..32]);
        rest = Some(Vec::from(&key[32..]))
    }

    (prefix, child, rest)
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
    C: BlockchainEvents<B> + 'static,
{
    pub async fn prove_current_block(da_mode: DaMode, client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let pallet_starknet_key = twox_128(PALLET_STARKNET);

        let mut storage_event_st = client
            .storage_changes_notification_stream(None, None)
            .expect("node has been initialized to prove state change, but can't read from notification stream");

        while let Some(storage_event) = storage_event_st.next().await {
            let mut diff = StateDiff::default();

            // Locate and encode the storage change
            for (_, storage_key, storage_val) in storage_event.changes.iter() {
                let (prefix_key, child_key, rest_key) = safe_split(&storage_key.0);
                if prefix_key == pallet_starknet_key {
                    if child_key == twox_128(STARKNET_NONCE) {
                        if let Some(val) = storage_val {
                            log::info!("encoding nonce change: {:?} {:?} {:?}", child_key, rest_key, val);
                            diff.nonces.insert(ContractAddress::from(rest_key.unwrap()), Nonce(val.as_slice()));
                        }
                    } else if child_key == twox_128(STARKNET_STORAGE) {
                        log::info!("encoding storage change: {:?} {:?}", child_key, rest_key);
                        // if let Some(val) = storage_val {
                        //     storage_diffs
                        //         .entry(child_key)
                        //         .and_modify(|v| v.push((rest_key, val.as_slice())))
                        //         .or_insert(vec![(rest_key, val.as_slice())]);
                        // }
                    } else if child_key == twox_128(STARKNET_CONTRACT_CLASS) {
                        log::info!("encoding class declaration: {:?} {:?}", child_key, rest_key);
                    } else if child_key == twox_128(STARKNET_CONTRACT_CLASS_HASH) {
                        log::info!("encoding replaced class: {:?} {:?}", child_key, rest_key);
                    }
                }
            }

            // let state_diff = utils::pre_0_11_0_state_diff(storage_diffs, nonces);

            // // Store the DA output from the SN OS
            // if let Err(db_err) = madara_backend.da().store_state_diff(&storage_event.block, state_diff) {
            //     log::error!("db err: {db_err}");
            // };

            match da_mode {
                DaMode::Validity => {
                    // Submit the StarkNet OS PIE
                    // TODO: Validity Impl
                    // run the Starknet OS with the Cairo VM
                    // extract the PIE from the Cairo VM run
                    // pass the PIE to `submit_pie` and zip/base64 internal
                    if let Ok(job_resp) = sharp::submit_pie("TODO") {
                        log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                        // Store the cairo job key
                        if let Err(db_err) =
                            madara_backend.da().update_cairo_job(&storage_event.block, job_resp.cairo_job_key)
                        {
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
