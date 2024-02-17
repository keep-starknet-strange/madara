#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Failed to initialize wallet: {0}")]
    EthersWallet(#[from] ethers::signers::WalletError),
    #[error("Failed to parse hex string: {0}")]
    ParseFromHex(#[from] rustc_hex::FromHexError),
    #[error("Undefined {0} contract address")]
    UndefinedContractAddress(&'static str),
    #[error("Failed to read config from file: {0}")]
    ReadFromFile(#[source] std::io::Error),
    #[error("Failed to decode from JSON: {0}")]
    JsonDecode(#[source] serde_json::Error),
}
