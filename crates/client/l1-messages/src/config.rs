use std::fs::File;
use std::path::PathBuf;

use ethers::types::Address;

use crate::error::L1MessagesConfigError;

#[derive(Clone, PartialEq, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1MessagesWorkerConfig {
    http_provider: String,
    contract_address: Address,
}

impl L1MessagesWorkerConfig {
    pub fn new(http_provider: String, contract_address: Address) -> Self {
        Self { http_provider, contract_address }
    }

    pub fn new_from_file(path: &PathBuf) -> Result<Self, L1MessagesConfigError> {
        let file = File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn provider(&self) -> &String {
        &self.http_provider
    }

    pub fn contract_address(&self) -> &Address {
        &self.contract_address
    }
}
