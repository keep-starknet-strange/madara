use std::fs::File;
use std::path::PathBuf;

use mc_eth_client::config::{EthereumProviderConfig, EthereumWalletConfig, StarknetContracts};
use serde::{Deserialize, Serialize};

use crate::{DaError, DaMode};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EthereumDaConfig {
    #[serde(default)]
    pub provider: EthereumProviderConfig,
    #[serde(default)]
    pub wallet: Option<EthereumWalletConfig>,
    #[serde(default)]
    pub contracts: StarknetContracts,
    #[serde(default)]
    pub mode: DaMode,
}

impl TryFrom<&PathBuf> for EthereumDaConfig {
    type Error = DaError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(DaError::FailedOpeningConfig)?;
        serde_json::from_reader(file).map_err(DaError::FailedParsingConfig)
    }
}
