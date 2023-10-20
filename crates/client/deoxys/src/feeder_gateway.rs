//! A simple HTTP client that simplifies the process of interacting with a Starknet Feeder Gateway.

use core::fmt;

use starknet_core::types::{BlockId, BlockTag};

/// The configuration passed to a [`FeederGatewayClient`].
pub struct FeederGatewayClientConfig {
    /// The base URL of the Feeder gateway.
    pub base_url: Box<str>,
}

/// An error that can occur when interacting with a [`FeederGatewayClient`].
pub enum FeederGatewayError {
    /// An error occured while transporting the request or the response over HTTP.
    Http(reqwest::Error),
    /// The gateway returned an error.
    Gateway,
    /// The gateway behaved in an unexpected way.
    UnexpectedBehavior,
}

/// A simple HTTP client that simplifies the process of interacting with a Starknet Feeder Gateway.
pub struct FeederGatewayClient {
    /// The raw HTTP client we're using to create our requests.
    client: reqwest::Client,

    /// The base URL of the Feeder Gateway.
    base_url: Box<str>,
}

impl FeederGatewayClient {
    /// Creates a new [`FeederGatewayClient`] with the given configuration.
    pub fn new(config: FeederGatewayClientConfig) -> Self {
        Self { client: reqwest::Client::new(), base_url: config.base_url }
    }

    /// TODO: doc
    pub async fn get_block(&self, id: BlockId) -> Result<String, FeederGatewayError> {
        let url = format!("{}/getBlock?{}", self.base_url, QueryBlockId(id));
        let response = self.client.request(reqwest::Method::GET, url).send().await.map_err(FeederGatewayError::Http)?;

        if response.status() != reqwest::StatusCode::OK {
            return Err(FeederGatewayError::Gateway);
        }

        response.json().await.map_err(|_| FeederGatewayError::UnexpectedBehavior)
    }
}

/// An implementation of [`fmt::Display`] that displays a [`BlockId`] as an URL query parameter.
struct QueryBlockId(BlockId);

impl fmt::Display for QueryBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            BlockId::Number(number) => write!(f, "blockNumber={number}"),
            BlockId::Hash(hash) => write!(f, "blockHash={hash}"),
            BlockId::Tag(BlockTag::Latest) => write!(f, "latest"),
            BlockId::Tag(BlockTag::Pending) => write!(f, "pending"),
        }
    }
}
