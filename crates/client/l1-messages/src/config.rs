use std::fs::File;
use std::path::PathBuf;

use ethers::types::Address;
use serde::{Deserialize, Deserializer};
use url::Url;

use crate::error::L1MessagesConfigError;

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

    pub fn new_from_file(path: &PathBuf) -> Result<Self, L1MessagesConfigError> {
        let file = File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn provider(&self) -> &Url {
        &self.http_provider
    }

    pub fn contract_address(&self) -> &Address {
        &self.contract_address
    }
}
