//! Ethereum configuration primitives and base Ethers client constructors.
//!
//! If you need to interact with Ethereum in your service, the suggested w/f is the following:
//!     - Import Ethers bindings for particular contract interface(s) from the Zaun crate;
//!     - Use config type from this crate as is or extend it, inheriting top-level sections, so that
//!       different services can reuse a single JSON file;
//!     - Construct Ethers client (middleware) using the config and initialize the high-level
//!       bindings imported from Zaun.

pub mod config;
pub mod error;

use std::time::Duration;

use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};

use crate::config::{EthereumClientConfig, EthereumProviderConfig, EthereumWalletConfig};
use crate::error::Error;

impl TryFrom<EthereumProviderConfig> for Provider<Http> {
    type Error = Error;

    fn try_from(config: EthereumProviderConfig) -> Result<Self, Self::Error> {
        match config {
            EthereumProviderConfig::Http(config) => {
                let mut provider = Provider::<Http>::try_from(config.rpc_endpoint).map_err(Error::ProviderUrlParse)?;

                if let Some(poll_interval_ms) = config.tx_poll_interval_ms {
                    provider = provider.interval(Duration::from_millis(poll_interval_ms));
                }

                Ok(provider)
            }
        }
    }
}

impl TryFrom<EthereumWalletConfig> for LocalWallet {
    type Error = Error;

    fn try_from(config: EthereumWalletConfig) -> Result<Self, Self::Error> {
        match config {
            EthereumWalletConfig::Local(config) => Ok(config
                .private_key
                .parse::<LocalWallet>()
                .map_err(Error::PrivateKeyParse)?
                .with_chain_id(config.chain_id)),
        }
    }
}

impl TryFrom<EthereumClientConfig> for SignerMiddleware<Provider<Http>, LocalWallet> {
    type Error = Error;

    fn try_from(config: EthereumClientConfig) -> Result<Self, Self::Error> {
        let provider: Provider<Http> = config.provider.try_into()?;
        let wallet: LocalWallet = config.wallet.unwrap_or_default().try_into()?;
        Ok(SignerMiddleware::new(provider, wallet))
    }
}
