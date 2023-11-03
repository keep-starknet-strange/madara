use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

use crate::DaMode;

pub const DEFAULT_DA_SERVER_ADDRESS: &str = "http://127.0.0.1:5888";

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct NearConfig {
    #[serde(default = "default_da_server_address")]
    pub da_server_address: String,

    pub da_server_config: Option<near_da_http_api_data::ConfigureClientRequest>,

    #[serde(default)]
    pub mode: DaMode,
}

fn default_da_server_address() -> String {
    DEFAULT_DA_SERVER_ADDRESS.to_string()
}

impl TryFrom<&PathBuf> for NearConfig {
    type Error = String;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}
