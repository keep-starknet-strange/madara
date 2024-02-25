use mc_eth_client::config::{EthereumProviderConfig, EthereumWalletConfig, StarknetContracts};
use serde::{Deserialize, Serialize};

use crate::DaMode;

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
