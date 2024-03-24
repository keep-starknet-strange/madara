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

use url::Url;
use alloy::{
    network::{Ethereum, EthereumSigner},
    providers::{layers::SignerProvider, ProviderBuilder, RootProvider},
    rpc::client::RpcClient,
    signers::wallet::LocalWallet,
    transports::http::Http
};
use k256::SecretKey;

use crate::config::{EthereumClientConfig, EthereumProviderConfig, EthereumWalletConfig};
use crate::error::Error;

impl TryFrom<EthereumProviderConfig> for RootProvider<Ethereum, Http<reqwest::Client>>  {
    type Error = Error;

    fn try_from(config: EthereumProviderConfig) -> Result<Self, Self::Error> {
        match config {
            EthereumProviderConfig::Http(config) => {
                let provider = ProviderBuilder::new()
                    .on_client(RpcClient::new_http(Url::parse(&config.rpc_endpoint).map_err(Error::ProviderUrlParse)?));

                // if let Some(poll_interval_ms) = config.tx_poll_interval_ms {
                //     provider = provider.interval(Duration::from_millis(poll_interval_ms));
                // }

                Ok(provider)
            }
        }
    }
}

impl TryFrom<EthereumWalletConfig> for LocalWallet {
    type Error = Error;

    fn try_from(config: EthereumWalletConfig) -> Result<Self, Self::Error> {
        match config {
            EthereumWalletConfig::Local(config) => {
                let key_str =
                    config.private_key.split("0x").last().ok_or(Error::PrivateKeyParse)?.trim();
                let key_hex = alloy::primitives::hex::decode(key_str).map_err(Error::FromHexError)?;
                let private_key = SecretKey::from_bytes((&key_hex[..]).into())
                    .map_err(|_| Error::DeserializePrivateKeyError)?;

                let wallet: LocalWallet = private_key
                    .clone()
                    .into();
                // wallet.with_chain_id(Some(config.chain_id));
                Ok(wallet)
            }
        }
    }
}

impl TryFrom<EthereumClientConfig> for SignerProvider<Ethereum, Http<reqwest::Client>, RootProvider<Ethereum, Http<reqwest::Client>>, EthereumSigner> {
    type Error = Error;

    fn try_from(config: EthereumClientConfig) -> Result<Self, Self::Error> {
        let wallet: LocalWallet = config.wallet.unwrap_or_default().try_into()?;
        let provider_config = config.provider;
        match provider_config {
            EthereumProviderConfig::Http(provider_config) => {
                let rpc_client = RpcClient::new_http(Url::parse(&provider_config.rpc_endpoint).map_err(Error::ProviderUrlParse)?);
                let provider_with_signer = ProviderBuilder::<_, Ethereum>::new()
                    .signer(EthereumSigner::from(wallet))
                    .network::<Ethereum>()
                    .provider(RootProvider::new(rpc_client));
                Ok(provider_with_signer)
            }
        }
    }
}
