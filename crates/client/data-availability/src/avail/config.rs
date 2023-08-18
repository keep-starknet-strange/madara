use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

const MADARA_DEFAULT_APP_ID: u32 = 0;
const AVAIL_VALIDATE_CODEGEN: bool = true;
const AVAIL_WS: &str = "wss://kate.avail.tools:443/ws";
const AVAIL_DEFAULT_SEED: &str = "//Alice";


#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct AvailConfig {
    #[serde(default = "default_ws")]
    pub ws_provider: String,
    #[serde(default = "default_app_id")]
    pub app_id: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_validate_codegen")]
    pub validate_codegen: bool,
    #[serde(default = "default_seed")]
    pub seed: String,
}

impl AvailConfig {
    pub fn new(path: &PathBuf) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}

fn default_ws() -> String {
    AVAIL_WS.to_string()
}


fn default_mode() -> String {
    "sovereign".to_string()
}

fn default_app_id() -> u32 {
    MADARA_DEFAULT_APP_ID
}

fn default_validate_codegen() -> bool {
    AVAIL_VALIDATE_CODEGEN
}

fn default_seed() -> String {
    AVAIL_DEFAULT_SEED.to_string()
}

impl Default for AvailConfig {
    fn default() -> Self {
        Self {
            ws_provider: default_ws(),
            app_id: default_app_id(),
            mode: default_mode(),
            validate_codegen: default_validate_codegen(),
            seed: default_seed(),
        }
    }
}
