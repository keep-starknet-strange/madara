use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

use crate::DaMode;

pub const DEFAULT_RPC_ADDRESS: &str = "127.0.0.1:3030";
pub const DEFAULT_SEQUENCER_ACCOUNT_ID: &str = "test.near";

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct NearConfig {
    #[serde(default = "default_rpc")]
    pub rpc_address: String,

    pub contract_account_id: String,

    #[serde(default = "default_sequencer_account_id")]
    pub sequencer_account_id: String,

    pub sequencer_key: String,

    #[serde(default)]
    pub mode: DaMode,
}

fn default_rpc() -> String {
    format!("http://{DEFAULT_RPC_ADDRESS}")
}

fn default_sequencer_account_id() -> String {
    DEFAULT_SEQUENCER_ACCOUNT_ID.to_string()
}

impl TryFrom<&PathBuf> for NearConfig {
    type Error = String;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}
