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
use sc_client_api::client::BlockchainEvents;
use serde::Deserialize;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;

pub type StorageWrites<'a> = Vec<(&'a [u8], &'a [u8])>;

pub struct DataAvailabilityWorker<B, C>(PhantomData<(B, C)>);

#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum DaLayer {
    Celestia,
    Ethereum,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Default)]
pub enum DaMode {
    #[serde(rename = "validity")]
    Validity,
    #[serde(rename = "volition")]
    Volition,
    #[serde(rename = "validium")]
    #[default]
    Validium,
}

#[async_trait]
pub trait DaClient {
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
        let mut storage_event_st = client.storage_changes_notification_stream(None, None).unwrap();

        while let Some(storage_event) = storage_event_st.next().await {
            // Locate and encode the storage change
            let mut _deployed_contracts: Vec<String> = Vec::new();
            let mut nonces: HashMap<&[u8], &[u8]> = HashMap::new();
            let mut storage_diffs: HashMap<&[u8], StorageWrites> = HashMap::new();

            // Locate and encode the storage change
            for event in storage_event.changes.iter() {
                let mut prefix = event.1.0.as_slice();
                let mut key: &[u8] = &[];
                if prefix.len() > 32 {
                    let raw_split = prefix.split_at(32);
                    prefix = raw_split.0;
                    key = raw_split.1;
                }

                if prefix == *utils::SN_NONCE_PREFIX {
                    if let Some(data) = event.2 {
                        nonces.insert(key, data.0.as_slice());
                    }
                }

                if prefix == *utils::SN_STORAGE_PREFIX {
                    if let Some(data) = event.2 {
                        // first 32 bytes = contract address, second 32 bytes = storage variable
                        let write_split = key.split_at(32);

                        storage_diffs
                            .entry(write_split.0)
                            .and_modify(|v| v.push((write_split.1, data.0.as_slice())))
                            .or_insert(vec![(write_split.1, data.0.as_slice())]);
                    }
                }
            }

            let state_diff = utils::pre_0_11_0_state_diff(storage_diffs, nonces);

            // Store the DA output from the SN OS
            if let Err(db_err) = madara_backend.da().store_state_diff(&storage_event.block, state_diff) {
                log::error!("db err: {db_err}");
            };

            match da_mode {
                DaMode::Validity => {
                    // Submit the StarkNet OS PIE
                    if let Ok(job_resp) = sharp::submit_pie("test") {
                        log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                        // Store the cairo job key
                        let _res = madara_backend.da().update_cairo_job(&storage_event.block, job_resp.cairo_job_key);
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
    // pub async fn update_state(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
    pub async fn update_state(da_client: impl DaClient, client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last written state
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
                DaMode::Validium => match madara_backend.da().state_diff(&notification.hash) {
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
