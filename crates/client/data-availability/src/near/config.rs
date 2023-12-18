use std::fs::File;
use std::path::PathBuf;

use near_da_rpc::near::config::Network;
use near_da_rpc::Namespace;
use serde::Deserialize;

use crate::DaMode;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct NearConfig {
    pub account_id: String,
    pub secret_key: String,
    pub contract_id: String,
    pub network: Network,
    pub namespace: Namespace,

    #[serde(default)]
    pub mode: DaMode,
}

impl TryFrom<&PathBuf> for NearConfig {
    type Error = String;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}
