/// Represents various error types that can occur during state synchronization or interaction with
/// L1/L2 chains.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Open configuration file failed")]
    OpenConfFailed,

    #[error("Deserialize StateSyncConfig has error {0}")]
    DeserializeConf(String),

    #[error("Failed to parse HTTP provider URL: {0}")]
    UrlParser(#[from] url::ParseError),

    #[error("Http provider has error: {0}")]
    Provider(#[from] ethers::providers::ProviderError),

    #[error("Parse address from string has error: {0}")]
    ParseAddress(String),

    #[error("The data is already present in the chain")]
    AlreadyInChain,

    #[error("unknown block or data on the chain")]
    UnknownBlock,

    #[error("Commit block chain storage data to backend has error: {0}")]
    CommitBlockChain(#[from] sp_blockchain::Error),

    #[error("Commit storage data to madara backend has error: {0}")]
    CommitMadara(String),

    #[error("A StarkNet fact corresponding to memory pages that cannot be found on l1")]
    BadStarknetFact,

    #[error("Reached the maximum number of connecting L1 retry attempts")]
    MaxRetryReached,

    #[error("The query from L1 has a response, but the returned value is empty")]
    EmptyValue,

    #[error("Decode an event from L1. has error: {0}")]
    L1EventDecode(#[from] ethers::abi::Error),

    #[error("The format of state update event on l1 is unknown")]
    UnknownStateUpdateEvent,

    #[error(
        "Can't Find starknet state transition fact for block_number: {block_number}, transaction_index: {tx_index}"
    )]
    FindFact { block_number: u64, tx_index: u64 },

    #[error("Convert some type from Felt252Wrapper has error: {0}")]
    TryFromFelt252Wrapper(#[from] mp_felt::Felt252WrapperError),
}
