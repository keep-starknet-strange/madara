//! Base Ethereum client configuration.
//!
//! Use it as is or reuse top-level fields in your config ("inherit")
//! in order to share the same configuration file.
//!
//! struct MyServiceConfig {
//!     pub provider: EthereumProviderConfig,
//!     pub wallet: Option<EthereumWalletConfig>,
//!     pub contracts: Option<StarknetContracts>,
//! }
//!
//! Default provider and wallet configurations are set for use with Anvil
//! - local Ethereum environment.

use std::fs::File;
use std::path::PathBuf;

use ethers::types::{Address, H160};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Default Anvil local endpoint
pub const DEFAULT_RPC_ENDPOINT: &str = "http://127.0.0.1:8545";
/// Default Anvil chain ID
pub const DEFAULT_CHAIN_ID: u64 = 31337;
/// Default private key derived from starting Anvil as follows:
/// anvil -b 5 --config-out $BUILD_DIR/anvil.json
/// PRE_PRIVATE=$(jq -r '.private_keys[0]' $BUILD_DIR/anvil.json)
pub const DEFAULT_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

pub const DEFAULT_API_URL: &str = "https://api.dev.pragma.build/node/v1/data/";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EthereumClientConfig {
    #[serde(default)]
    pub provider: EthereumProviderConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet: Option<EthereumWalletConfig>,
    #[serde(default)]
    pub contracts: StarknetContracts,
    #[serde(default)]
    pub oracle: OracleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EthereumProviderConfig {
    Http(HttpProviderConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EthereumWalletConfig {
    Local(LocalWalletConfig),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StarknetContracts {
    pub core_contract: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_pages_contract: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProviderConfig {
    #[serde(default = "default_rpc_endpoint")]
    pub rpc_endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_poll_interval_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_price_poll_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "oracle_name", content = "config")]
pub enum OracleConfig {
    Pragma(PragmaOracle),
}

impl OracleConfig {
    pub fn get_fetch_url(&self, base: String, quote: String) -> String {
        match self {
            OracleConfig::Pragma(pragma_oracle) => pragma_oracle.get_fetch_url(base, quote),
        }
    }

    pub fn get_api_key(&self) -> &String {
        match self {
            OracleConfig::Pragma(oracle) => &oracle.api_key,
        }
    }

    pub fn is_in_bounds(&self, price: u128) -> bool {
        match self {
            OracleConfig::Pragma(oracle) => oracle.price_bounds.low <= price && price <= oracle.price_bounds.high,
        }
    }
}

/// Price bounds for the oracle
/// If the price is outside of these bounds, it will not be used
/// The bounds are denominated in the quote currency so in FRI here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriceBounds {
    pub low: u128,
    pub high: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PragmaOracle {
    #[serde(default = "default_api_url")]
    pub api_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub aggregation_method: AggregationMethod,
    #[serde(default)]
    pub interval: Interval,
    #[serde(default)]
    pub price_bounds: PriceBounds,
}

impl PragmaOracle {
    fn get_fetch_url(&self, base: String, quote: String) -> String {
        format!(
            "{}{}/{}?interval={:?}&aggregation={:?}",
            self.api_url, base, quote, self.interval, self.aggregation_method
        )
    }
}

/// Supported Aggregation Methods
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum AggregationMethod {
    #[serde(rename = "median")]
    Median,
    #[serde(rename = "mean")]
    Mean,
    #[serde(rename = "twap")]
    #[default]
    Twap,
}

/// Supported Aggregation Intervals
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum Interval {
    #[serde(rename = "1min")]
    OneMinute,
    #[serde(rename = "15min")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    #[default]
    TwoHours,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWalletConfig {
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    #[serde(default = "default_private_key")]
    pub private_key: String,
}

fn default_rpc_endpoint() -> String {
    DEFAULT_RPC_ENDPOINT.into()
}

fn default_api_url() -> String {
    DEFAULT_API_URL.into()
}

fn default_chain_id() -> u64 {
    DEFAULT_CHAIN_ID
}

fn default_private_key() -> String {
    DEFAULT_PRIVATE_KEY.to_string()
}

impl Default for PragmaOracle {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            api_key: String::default(),
            aggregation_method: AggregationMethod::Median,
            interval: Interval::OneMinute,
            price_bounds: Default::default(),
        }
    }
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self::Pragma(PragmaOracle::default())
    }
}

impl Default for HttpProviderConfig {
    fn default() -> Self {
        Self { rpc_endpoint: default_rpc_endpoint(), tx_poll_interval_ms: None, gas_price_poll_ms: None }
    }
}

impl Default for EthereumProviderConfig {
    fn default() -> Self {
        Self::Http(HttpProviderConfig::default())
    }
}

impl EthereumProviderConfig {
    pub fn rpc_endpoint(&self) -> &String {
        match self {
            Self::Http(config) => &config.rpc_endpoint,
        }
    }

    pub fn gas_price_poll_ms(&self) -> Option<u64> {
        match self {
            Self::Http(config) => config.gas_price_poll_ms,
        }
    }
}

impl Default for LocalWalletConfig {
    fn default() -> Self {
        Self { chain_id: default_chain_id(), private_key: default_private_key() }
    }
}

impl Default for EthereumWalletConfig {
    fn default() -> Self {
        Self::Local(LocalWalletConfig::default())
    }
}

fn parse_contract_address(string: &str) -> Result<H160, Error> {
    string.parse().map_err(Error::ContractAddressParse)
}

impl StarknetContracts {
    pub fn core_contract(&self) -> Result<Address, Error> {
        parse_contract_address(&self.core_contract)
    }

    pub fn verifier_contract(&self) -> Result<Address, Error> {
        self.verifier_contract
            .as_ref()
            .map(|s| parse_contract_address(s))
            .ok_or(Error::ContractAddressUndefined("verifier"))?
    }

    pub fn memory_pages_contract(&self) -> Result<Address, Error> {
        self.memory_pages_contract
            .as_ref()
            .map(|s| parse_contract_address(s))
            .ok_or(Error::ContractAddressUndefined("memory pages"))?
    }
}

impl EthereumClientConfig {
    pub fn from_json_file(path: &PathBuf) -> Result<Self, Error> {
        let file = File::open(path).map_err(Error::ConfigReadFromFile)?;
        serde_json::from_reader(file).map_err(Error::ConfigDecodeFromJson)
    }
}
