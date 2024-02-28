#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse provider URL: {0}")]
    ProviderUrlParse(#[source] url::ParseError),
    #[error("Failed to parse private key: {0}")]
    PrivateKeyParse(#[source] ethers::signers::WalletError),
    #[error("Failed to parse contract address: {0}")]
    ContractAddressParse(#[source] rustc_hex::FromHexError),
    #[error("Undefined {0} contract address")]
    ContractAddressUndefined(&'static str),
    #[error("Failed to read config from file: {0}")]
    ConfigReadFromFile(#[source] std::io::Error),
    #[error("Failed to decode from JSON: {0}")]
    ConfigDecodeFromJson(#[source] serde_json::Error),
}
