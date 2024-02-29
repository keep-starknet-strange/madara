pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::providers::{Http, Provider};
use ethers::types::{I256, U256};
use starknet_core_contract_client::interfaces::StarknetSovereignContract;

use crate::{DaClient, DaError, DaMode};

#[derive(Clone, Debug)]
pub struct EthereumDaClient {
    core_contract: StarknetSovereignContract<Provider<Http>>,
    mode: DaMode,
}

#[async_trait]
impl DaClient for EthereumDaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<(), anyhow::Error> {
        log::debug!("State diff: {:?}", state_diff);
        Ok(())
    }

    async fn last_published_state(&self) -> Result<I256, anyhow::Error> {
        self.core_contract
            .state_block_number()
            .call()
            .await
            .map_err(|e| DaError::FailedDataSubmission(e.into()))
            .map_err(Into::into)
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "ethereum".into())].iter().cloned().collect()
    }
}

impl TryFrom<config::EthereumDaConfig> for EthereumDaClient {
    type Error = DaError;

    fn try_from(conf: config::EthereumDaConfig) -> Result<Self, Self::Error> {
        // NOTE: only sovereign mode is supported (for now)
        // In sovereign mode both proof and state diff are populated on-chain
        // without verification. A full Madara node should be able to index
        // from scratch using just that info: verify proof -> apply diff
        if conf.mode != DaMode::Sovereign {
            return Err(DaError::UnsupportedMode(conf.mode));
        }

        let address = conf.contracts.core_contract().map_err(|e| DaError::FailedConversion(e.into()))?;
        let provider =
            Provider::<Http>::try_from(conf.provider).map_err(|e| DaError::FailedBuildingClient(e.into()))?;
        let core_contract = StarknetSovereignContract::new(address, Arc::new(provider));

        Ok(Self { mode: conf.mode, core_contract })
    }
}
