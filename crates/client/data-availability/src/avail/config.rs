use serde::Deserialize;

use crate::DaMode;

const DEFAULT_AVAIL_WS: &str = "ws://127.0.0.1:9945";
const DEFAULT_APP_ID: u32 = 0;
const DEFAULT_AVAIL_VALIDATE_CODEGEN: bool = false;
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
    #[serde(default)]
    pub mode: DaMode,
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

impl Default for AvailConfig {
    fn default() -> Self {
        Self {
            ws_provider: default_ws(),
            app_id: default_app_id(),
            mode: DaMode::default(),
            validate_codegen: default_validate_codegen(),
            seed: default_seed(),
        }
    }
}
