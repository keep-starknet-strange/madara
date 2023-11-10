use mp_felt::Felt252WrapperError;

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum L1MessagesConfigError {
    #[error("File with L1 Messages Worker config not found")]
    FileNotFound(String),
    #[error("Failed to deserialize L1 Messages Worker Config from config file")]
    InvalidFile(String),
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum L1MessagesWorkerError {
    #[error("Failed to initialize L1 Messages Worker based on provided Config")]
    ConfigError,
    #[error("Failed to convert transaction via Runtime API")]
    ConvertTransactionRuntimeApiError,
    #[error("Message from L1 has been already processed")]
    L1MessageAlreadyProcessed,
    #[error("Failed to read/write into Offchain Storage")]
    OffchainStorageError,
    #[error("Failed to use Runtime API")]
    RuntimeApiError,
    #[error("Failed to submit transaction into Transaction Pool")]
    SubmitTxError,
    #[error("Failed to convert L1 Message into Fee")]
    ToFeeError,
    #[error("Failed to convert L1 Message into L2 Transaction")]
    ToTransactionError(String),
}

impl From<Felt252WrapperError> for L1MessagesWorkerError {
    fn from(e: Felt252WrapperError) -> Self {
        L1MessagesWorkerError::ToTransactionError(format!("{:?}", e))
    }
}
