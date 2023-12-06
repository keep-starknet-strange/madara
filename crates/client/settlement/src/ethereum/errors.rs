use super::client::LocalMiddleware;

/// Ethereum client error type.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Failed to parse HTTP provider URL: {0}")]
    UrlParser(#[from] url::ParseError),

    #[error("Failed to initialize local wallet from private key: {0}")]
    LocalWallet(#[from] ethers::signers::WalletError),

    #[error("Failed to parse contract address: {0}")]
    HexParser(#[from] rustc_hex::FromHexError),

    #[error("Error while interacting with contract: {0}")]
    Contract(#[from] ethers::contract::ContractError<LocalMiddleware>),

    #[error("HTTP provider error: {0}")]
    Provider(#[from] ethers::providers::ProviderError),

    #[error("Failed to get transaction receipt")]
    MissingTransactionRecepit,
}

pub type Result<T> = std::result::Result<T, Error>;
