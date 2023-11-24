use mp_felt::Felt252WrapperError;

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
    #[error("File with L1 Messages Worker config not found")]
    FileNotFound(#[from] std::io::Error),
    #[error("Failed to deserialize L1 Messages Worker Config from config file")]
    InvalidFile(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum L1MessagesWorkerError {
    #[error("Failed to initialize L1 Messages Worker based on provided Config")]
    ConfigError,
    #[error("Failed to convert transaction via Runtime API")]
    ConvertTransactionRuntimeApiError,
    #[error("Madara Messaging DB Error: `{0}`")]
    DatabaseError(String),
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
    ToTransactionError(#[from] L1EventToTransactionError),
}
