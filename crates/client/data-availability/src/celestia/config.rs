use serde::Deserialize;

use crate::{DaError, DaMode};

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
    #[serde(default)]
    pub mode: DaMode,
}

impl TryFrom<&PathBuf> for CelestiaConfig {
    type Error = DaError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| DaError::FailedOpeningConfig(e))?;
        serde_json::from_reader(file).map_err(|e| DaError::FailedParsingConfig(e))
    }
}

fn default_http() -> String {
    format!("http://{DEFAULT_CELESTIA_NODE}")
}

fn default_ws() -> String {
    format!("ws://{DEFAULT_CELESTIA_NODE}")
}

fn default_nid() -> String {
    DEFAULT_NID.to_string()
}

impl Default for CelestiaConfig {
    fn default() -> Self {
        Self {
            http_provider: default_http(),
            ws_provider: default_ws(),
            nid: default_nid(),
            mode: DaMode::default(),
            auth_token: None,
        }
    }
}
