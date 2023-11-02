use std::fs::File;
use std::path::PathBuf;

pub const DEFAULT_ETHEREUM_NODE: &str = "127.0.0.1:8545";
pub const DEFAULT_CONTRACT_ADDRESS: &str = "0x5fbdb2315678afecb367f032d93f642f64180aa3";

#[derive(Clone, PartialEq, serde::Deserialize, Debug)]
pub struct L1MessagesWorkerConfig {
    #[serde(default = "default_http")]
    pub http_provider: String,
    #[serde(default = "default_contract_address")]
    pub contract_address: String,
}

impl TryFrom<&PathBuf> for L1MessagesWorkerConfig {
    type Error = String;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("Error opening L1 Messages Worker config: {:?}", e))?;
        serde_json::from_reader(file).map_err(|e| format!("Error parsing L1 Messages Worker config: {:?}", e))
    }
}

fn default_http() -> String {
    format!("http://{DEFAULT_ETHEREUM_NODE}")
}

fn default_contract_address() -> String {
    DEFAULT_CONTRACT_ADDRESS.to_string()
}

impl Default for L1MessagesWorkerConfig {
    fn default() -> Self {
        Self { http_provider: default_http(), contract_address: default_contract_address() }
    }
}
