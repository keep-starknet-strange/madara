use std::fs::File;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{DaError, DaMode};

pub const DEFAULT_ETHEREUM_NODE: &str = "127.0.0.1:8545";
// default key derived from starting anvil as follows:
// anvil -b 5 --config-out $BUILD_DIR/anvil.json
// PRE_PRIVATE=$(jq -r '.private_keys[0]' $BUILD_DIR/anvil.json)
pub const DEFAULT_SEQUENCER_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
pub const DEFAULT_STARKNET_CORE_CONTRACTS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
pub const DEFAULT_CHAIN_ID: u64 = 31337;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct EthereumConfig {
    #[serde(default = "default_http")]
    pub http_provider: String,
    #[serde(default = "default_core_contracts")]
    pub core_contracts: String,
    #[serde(default = "default_sequencer_key")]
    pub sequencer_key: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    #[serde(default)]
    pub mode: DaMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_interval_ms: Option<u64>,
}

impl TryFrom<&PathBuf> for EthereumConfig {
    type Error = DaError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(DaError::FailedOpeningConfig)?;
        serde_json::from_reader(file).map_err(DaError::FailedParsingConfig)
    }
}

fn default_http() -> String {
    format!("http://{DEFAULT_ETHEREUM_NODE}")
}

fn default_core_contracts() -> String {
    DEFAULT_STARKNET_CORE_CONTRACTS.to_string()
}

fn default_sequencer_key() -> String {
    DEFAULT_SEQUENCER_KEY.to_string()
}

fn default_chain_id() -> u64 {
    DEFAULT_CHAIN_ID
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self {
            http_provider: default_http(),
            mode: DaMode::default(),
            core_contracts: default_core_contracts(),
            sequencer_key: default_sequencer_key(),
            chain_id: default_chain_id(),
            poll_interval_ms: None,
        }
    }
}
