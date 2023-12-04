use std::fs::File;
use std::path::PathBuf;

use ethers::types::Address;
use rustc_hex::FromHexError;
use serde::{Deserialize, Deserializer};
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum L1MessagesWorkerConfigError {
    #[error("File with L1 Messages Worker config not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("Failed to deserialize L1 Messages Worker Config from config file: {0}")]
    InvalidFile(#[from] serde_json::Error),
    #[error("Invalid Ethereum Provided Url: {0}")]
    InvalidProviderUrl(#[from] url::ParseError),
    #[error("Invalid L1 Contract Address: {0}")]
    InvalidContractAddress(#[from] FromHexError),
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1MessagesWorkerConfig {
    #[serde(deserialize_with = "deserialize_url")]
    http_provider: Url,
    contract_address: Address,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let endpoint: String = String::deserialize(deserializer)?;

    Url::parse(&endpoint).map_err(serde::de::Error::custom)
}

impl L1MessagesWorkerConfig {
    pub fn new(http_provider: Url, contract_address: Address) -> Self {
        Self { http_provider, contract_address }
    }

    pub fn new_from_file(path: &PathBuf) -> Result<Self, L1MessagesWorkerConfigError> {
        let file = File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn new_from_params(provider_url: &str, contract_address: &str) -> Result<Self, L1MessagesWorkerConfigError> {
        let http_provider = Url::parse(provider_url)?;
        let contract_address = contract_address.parse()?;
        Ok(Self { http_provider, contract_address })
    }

    pub fn provider(&self) -> &Url {
        &self.http_provider
    }

    pub fn contract_address(&self) -> &Address {
        &self.contract_address
    }
}
