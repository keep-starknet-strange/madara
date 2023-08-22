use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

use crate::DaMode;

pub const DEFAULT_CELESTIA_NODE: &str = "127.0.0.1:26658";
pub const DEFAULT_NID: &str = "Madara";

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct CelestiaConfig {
    #[serde(default = "default_http")]
    pub http_provider: String,
    #[serde(default = "default_ws")]
    pub ws_provider: String,
    #[serde(default = "default_nid")]
    pub nid: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default = "default_mode")]
    pub mode: DaMode,
}

impl CelestiaConfig {
    pub fn try_from_file(path: &PathBuf) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}

fn default_http() -> String {
    format!("http://{DEFAULT_CELESTIA_NODE}")
}

fn default_ws() -> String {
    format!("http://{DEFAULT_CELESTIA_NODE}")
}

fn default_nid() -> String {
    DEFAULT_NID.to_string()
}

fn default_mode() -> DaMode {
    DaMode::default()
}

impl Default for CelestiaConfig {
    fn default() -> Self {
        Self {
            http_provider: default_http(),
            ws_provider: default_ws(),
            nid: default_nid(),
            mode: default_mode(),
            auth_token: None,
        }
    }
}
