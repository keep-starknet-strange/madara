use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use avail_subxt::api::runtime_types::avail_core::AppId;
use avail_subxt::api::runtime_types::sp_core::bounded::bounded_vec::BoundedVec;
use celestia_types::Blob;
use ethers::types::U256;
use sp_core::H256;

// use crate::ethereum::EthereumDaClient;
use crate::avail::AvailClient;
use crate::celestia::CelestiaClient;

enum DAType {
    Avail,
    Celestia,
    Ethereum,
}
impl FromStr for DAType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Avail" => Ok(DAType::Avail),
            "Celestia" => Ok(DAType::Celestia),
            "Ethereum" => Ok(DAType::Ethereum),
            _ => Err(()),
        }
    }
}

pub enum DAArgs {
    Avail { ws_endpoint: String, app_id: AppId, avail_seed: String },
    Celestia { http_endpoint: String, ws_endpoint: String, auth_token: String, namespace: String },
    Ethereum { eth_node: String },
}

pub enum DAInput {
    Avail(BoundedVec<u8>),
    Celestia(Blob),
}

pub enum DAOutput {
    Avail { block_hash: H256 },
    Celestia { submitted_height: u64 },
    Ethereum,
}

#[async_trait]
pub trait DataAvailability: Send + Sync {
    fn new(da_config: &HashMap<String, String>) -> Result<Self>
    where
        Self: Sized;
    fn validate_args(da_config: &HashMap<String, String>) -> Result<DAArgs>
    where
        Self: Sized;
    fn format_state_diff(&self, state_diff: &[U256]) -> Result<DAInput>;
    async fn publish_data(&self, da_input: &DAInput) -> Result<DAOutput>;
    async fn verify_inclusion(&self, da_input: &DAInput, da_output: &DAOutput) -> Result<()>;
}

pub fn get_da_client(da_config: &HashMap<String, String>) -> Result<Arc<dyn DataAvailability + Send + Sync>> {
    let da = da_config.get("da_type").ok_or(anyhow::anyhow!("Invalid DA type argument."))?;
    let da_type = DAType::from_str(da).map_err(|_| anyhow::anyhow!("Invalid DA type."))?;
    let da_client: Arc<dyn DataAvailability + Send + Sync> = match da_type {
        DAType::Avail => Arc::new(AvailClient::new(da_config)?),
        DAType::Celestia => Arc::new(CelestiaClient::new(da_config)?),
        DAType::Ethereum => {
            todo!();
        }
    };
    Ok(da_client)
}

pub async fn submit_data_and_verify_inclusion(da_client: &Arc<dyn DataAvailability>, state_diff: &[U256]) {
    let da_input = match da_client.format_state_diff(state_diff) {
        Ok(input) => input,
        Err(e) => {
            log::error!("Failed to format state diff: {}", e);
            return;
        }
    };

    let da_output = match da_client.publish_data(&da_input).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to publish data: {}", e);
            return;
        }
    };

    if let Err(e) = da_client.verify_inclusion(&da_input, &da_output).await {
        log::error!("Failed to verify data inclusion: {}", e);
    }
}
