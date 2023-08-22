use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

use crate::DaMode;

const DEFAULT_AVAIL_WS: &str = "wss://kate.avail.tools:443/ws";
const DEFAULT_APP_ID: u32 = 0;
const DEFAULT_AVAIL_VALIDATE_CODEGEN: bool = true;
const DEFAULT_AVAIL_SEED: &str = "//Alice";

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct AvailConfig {
    #[serde(default = "default_ws")]
    pub ws_provider: String,
    #[serde(default = "default_app_id")]
    pub app_id: u32,
    #[serde(default = "default_validate_codegen")]
    pub validate_codegen: bool,
    #[serde(default = "default_seed")]
    pub seed: String,
    #[serde(default = "default_mode")]
    pub mode: DaMode,
}

impl AvailConfig {
    pub fn try_from_file(path: &PathBuf) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}

fn default_ws() -> String {
    DEFAULT_AVAIL_WS.to_string()
}

fn default_app_id() -> u32 {
    DEFAULT_APP_ID
}

fn default_validate_codegen() -> bool {
    DEFAULT_AVAIL_VALIDATE_CODEGEN
}

fn default_seed() -> String {
    DEFAULT_AVAIL_SEED.to_string()
}

fn default_mode() -> DaMode {
    DaMode::default()
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
