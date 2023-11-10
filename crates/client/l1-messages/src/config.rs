use std::fs::File;
use std::path::PathBuf;

use ethers::types::Address;

use crate::error::L1MessagesConfigError;

pub const DEFAULT_ETHEREUM_NODE: &str = "127.0.0.1:8545";
pub const DEFAULT_CONTRACT_ADDRESS: &str = "0x5fbdb2315678afecb367f032d93f642f64180aa3";

#[derive(Clone, PartialEq, serde::Deserialize, Debug)]
pub struct L1MessagesWorkerConfig {
    http_provider: String,
    contract_address: Address,
}

impl L1MessagesWorkerConfig {
    pub fn new(http_provider: String, contract_address: Address) -> Self {
        Self { http_provider, contract_address }
    }

    pub fn new_from_file(path: &PathBuf) -> Result<Self, L1MessagesConfigError> {
        let file = File::open(path).map_err(|_e| L1MessagesConfigError::FileNotFound(format!("{:?}", path)))?;
        serde_json::from_reader(file).map_err(|_e| L1MessagesConfigError::InvalidFile(format!("{:?}", path)))
    }

    pub fn get_provider(&self) -> &String {
        &self.http_provider
    }

    pub fn get_contract_address(&self) -> &Address {
        &self.contract_address
    }
}
