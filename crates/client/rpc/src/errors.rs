use jsonrpsee::types::error::{CallError, ErrorObject};

#[derive(thiserror::Error, Clone, Copy, Debug)]
pub enum StarknetRpcApiError {
    #[error("Failed to write transaction")]
    FailedToReceiveTxn = 1,
    #[error("Contract not found")]
    ContractNotFound = 20,
    #[error("Invalid message selector")]
    InvalidMessageSelector = 21,
    #[error("Invalid call data")]
    InvalidCallData = 22,
    #[error("Block not found")]
    BlockNotFound = 24,
    #[error("Transaction hash not found")]
    TxnHashNotFound = 25,
    #[error("Invalid transaction index in a block")]
    InvalidTxnIndex = 27,
    #[error("Class hash not found")]
    ClassHashNotFound = 28,
    #[error("Class already declared")]
    ClassAlreadyDeclared = 51,
    #[error("Requested page size is too big")]
    PageSizeTooBig = 31,
    #[error("There are no blocks")]
    NoBlocks = 32,
    #[error("The supplied continuation token is invalid or unknown")]
    InvalidContinuationToken = 33,
    #[error("Contract error")]
    ContractError = 40,
    #[error("Invalid contract class")]
    InvalidContractClass = 50,
    #[error("Too many storage keys requested")]
    ProofLimitExceeded = 10000,
    #[error("Too many keys provided in a filter")]
    TooManyKeysInFilter = 34,
    #[error("Internal server error")]
    InternalServerError = 500,
    #[error("Failed to fetch pending transactions")]
    FailedToFetchPendingTransactions = 38,
    #[error("Unimplemented method")]
    UnimplementedMethod = 501,
}

impl From<StarknetRpcApiError> for jsonrpsee::core::Error {
    fn from(err: StarknetRpcApiError) -> Self {
        jsonrpsee::core::Error::Call(CallError::Custom(ErrorObject::owned(err as i32, err.to_string(), None::<()>)))
    }
}
