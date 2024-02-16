use std::fs::File;
use std::path::PathBuf;
use serde::Deserialize;
use crate::DaMode;

pub const DEFAULT_SEQUENCER_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const DEFAULT_EIGENDA_CONTRACT: &str = "0xa3b1689Ab85409B15e07d2ED50A6EA9905074Ee5";
const DEFAULT_ETHEREUM_NODE: &str = "127.0.0.1:8545";
pub const DEFAULT_CHAIN_ID: u64 = 31337;
const DEFAULT_PROTO_PATH: &str = "/proto/disperser/disperser.proto";
// rollups will eventually run their own disperser
const DEFAULT_DISPERSER: &str = "disperser-goerli.eigenda.xyz:443";
const DEFAULT_QUORUM_ID: u32 = 0;
// thresholds set according to https://docs.eigenlayer.xyz/eigenda-guides/eigenda-rollup-user-guides/system-performance-and-customization
const DEFAULT_ADVERSARY_THRESHOLD: u32 = 33;
const DEFAULT_QUORUM_THRESHOLD: u32 = 80;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct EigenDaConfig {
    #[serde(default = "default_grpc")]
    pub grpc_provider: String,
    #[serde(default = "default_eth_rpc")]
    pub eth_rpc_provider: String,
    #[serde[default = "default_eigenda_contract"]]
    pub eigenda_contract: String,
    #[serde(default = "default_proto_path")]
    pub proto_path: String,
    #[serde(default = "default_sequencer_key")]
    pub sequencer_key: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
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
    format!("https://{DEFAULT_DISPERSER}")
}

fn default_eth_rpc() -> String {
    format!("http://{DEFAULT_ETHEREUM_NODE}")
}

fn default_eigenda_contract() -> String {
    format!("{DEFAULT_EIGENDA_CONTRACT}")
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

fn default_sequencer_key() -> String {
    DEFAULT_SEQUENCER_KEY.to_string()
}

fn default_chain_id() -> u64 {
    DEFAULT_CHAIN_ID
}

impl Default for EigenDaConfig {
    fn default() -> Self {
        Self {
            grpc_provider: default_grpc(),
            eth_rpc_provider: default_eth_rpc(),
            eigenda_contract: default_eigenda_contract(),
            proto_path: default_proto_path(),
            sequencer_key: default_sequencer_key(),
            chain_id: default_chain_id(),
            quorum_id: default_quorum_id(),
            adversary_threshold: default_adversary_threshold(),
            quorum_threshold: default_quorum_threshold(),
            mode: DaMode::default(),
        }
    }
}