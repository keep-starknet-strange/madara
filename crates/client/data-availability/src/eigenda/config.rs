use std::fs::File;
use std::path::PathBuf;
use serde::Deserialize;
use crate::DaMode;

const DEFAULT_PROTO_PATH: &str = "/proto/disperser/disperser.proto";
const DEFAULT_EIGEN_NODE: &str = "disperser-goerli.eigenda.xyz:443";    // local node?
const DEFAULT_QUORUM_ID: u32 = 0;
// thresholds set according to https://docs.eigenlayer.xyz/eigenda-guides/eigenda-rollup-user-guides/system-performance-and-customization
const DEFAULT_ADVERSARY_THRESHOLD: u32 = 33;
const DEFAULT_QUORUM_THRESHOLD: u32 = 80;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct EigenDaConfig {
    #[serde(default = "default_grpc")]
    pub grpc_provider: String,
    #[serde(default = "default_proto_path")]
    pub proto_path: String,
    //pub security_params: Vec<SecurityParams>,
    // can have multiple security params but only one per quorum
    // for now we have one set of security params
    #[serde(default = "default_quorum_id")]
    pub quorum_id: u32,
    #[serde(default = "default_adversary_threshold")]
    pub adversary_threshold: u32,
    #[serde(default = "default_quorum_threshold")]
    pub quorum_threshold: u32,
    #[serde(default)]
    pub mode: DaMode,
}

impl TryFrom<&PathBuf> for EigenDaConfig {
    type Error = String;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("error opening da config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing da config: {e}"))
    }
}

fn default_grpc() -> String {
    format!("https://{DEFAULT_EIGEN_NODE}")
}

fn default_proto_path() -> String {
    format!("{DEFAULT_PROTO_PATH}")
}

fn default_quorum_id() -> u32 {
    DEFAULT_QUORUM_ID
}

fn default_adversary_threshold() -> u32 {
    DEFAULT_ADVERSARY_THRESHOLD
}

fn default_quorum_threshold() -> u32 {
    DEFAULT_QUORUM_THRESHOLD
}

impl Default for EigenDaConfig {
    fn default() -> Self {
        Self {
            grpc_provider: default_grpc(),
            proto_path: default_proto_path(),
            quorum_id: default_quorum_id(),
            adversary_threshold: default_adversary_threshold(),
            quorum_threshold: default_quorum_threshold(),
            mode: DaMode::default(),
        }
    }
}