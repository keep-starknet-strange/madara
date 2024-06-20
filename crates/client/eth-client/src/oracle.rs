use serde::{Deserialize, Serialize};

pub const DEFAULT_API_URL: &str = "https://api.dev.pragma.build/node/v1/data/";

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

impl Default for OracleConfig {
    fn default() -> Self {
        Self::Pragma(PragmaOracle::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PragmaOracle {
    #[serde(default = "default_oracle_api_url")]
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

impl Default for PragmaOracle {
    fn default() -> Self {
        Self {
            api_url: default_oracle_api_url(),
            api_key: String::default(),
            aggregation_method: AggregationMethod::Median,
            interval: Interval::OneMinute,
            price_bounds: Default::default(),
        }
    }
}

impl PragmaOracle {
    fn get_fetch_url(&self, base: String, quote: String) -> String {
        format!(
            "{}{}/{}?interval={}&aggregation={}",
            self.api_url,
            base,
            quote,
            self.interval.as_str(),
            self.aggregation_method.as_str()
        )
    }
}

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

impl AggregationMethod {
    pub fn as_str(&self) -> &str {
        match self {
            AggregationMethod::Median => "median",
            AggregationMethod::Mean => "mean",
            AggregationMethod::Twap => "twap",
        }
    }
}

// Supported Aggregation Intervals
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

impl Interval {
    pub fn as_str(&self) -> &str {
        match self {
            Interval::OneMinute => "1min",
            Interval::FifteenMinutes => "15min",
            Interval::OneHour => "1h",
            Interval::TwoHours => "2h",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBounds {
    pub low: u128,
    pub high: u128,
}

impl Default for PriceBounds {
    fn default() -> Self {
        Self { low: 0, high: u128::MAX }
    }
}

fn default_oracle_api_url() -> String {
    DEFAULT_API_URL.into()
}
