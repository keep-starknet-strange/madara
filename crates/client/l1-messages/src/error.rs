use mc_db::DbError;
use mp_felt::Felt252WrapperError;
use rustc_hex::FromHexError;
use sp_api::ApiError;
use sp_runtime::DispatchError;
use url::ParseError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum L1EventToTransactionError {
    #[error("Failed to convert Calldata param from L1 Event: `{0}`")]
    InvalidCalldata(Felt252WrapperError),
    #[error("Failed to convert Contract Address from L1 Event: `{0}`")]
    InvalidContractAddress(Felt252WrapperError),
    #[error("Failed to convert Entrypoint Selector from L1 Event: `{0}`")]
    InvalidEntryPointSelector(Felt252WrapperError),
    #[error("Failed to convert Nonce param from L1 Event: `{0}`")]
    InvalidNonce(Felt252WrapperError),
}

#[derive(thiserror::Error, Debug)]
pub enum L1MessagesConfigError {
    #[error("File with L1 Messages Worker config not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("Failed to deserialize L1 Messages Worker Config from config file: {0}")]
    InvalidFile(#[from] serde_json::Error),
    #[error("Invalid Ethereum Provided Url: {0}")]
    InvalidProviderUrl(#[from] url::ParseError),
    #[error("Invalid L1 Contract Address: {0}")]
    InvalidContractAddress(#[from] FromHexError),
    #[error("Missing Ethereum Provided Url")]
    MissingProviderUrl,
    #[error("Missing L1 Contract Address")]
    MissingContractAddress,
}

#[derive(thiserror::Error, Debug)]
pub enum L1MessagesWorkerError {
    #[error("Failed to initialize L1 Messages Worker based on provided Config: `{0}`")]
    ConfigError(#[from] ParseError),
    #[error("Failed to convert transaction via Runtime API: `{0}`")]
    ConvertTransactionRuntimeApiError(ApiError),
    #[error("Failed to Dispatch Runtime API")]
    RuntimeApiDispatchError(DispatchError),
    #[error("Madara Messaging DB Error: `{0}`")]
    DatabaseError(#[from] DbError),
    #[error("Message from L1 has been already processed, nonce: `{0}`")]
    L1MessageAlreadyProcessed(u64),
    #[error("Failed to use Runtime API: `{0}`")]
    RuntimeApiError(ApiError),
    #[error("Failed to submit transaction into Transaction Pool")]
    SubmitTxError,
    #[error("Failed to convert L1 Message into Fee")]
    ToFeeError,
    #[error("Failed to convert L1 Message into L2 Transaction: `{0}`")]
    ToTransactionError(#[from] L1EventToTransactionError),
}
