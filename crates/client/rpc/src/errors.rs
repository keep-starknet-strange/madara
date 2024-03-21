use jsonrpsee::types::error::{CallError, ErrorObject};

// Comes from the RPC Spec:
// https://github.com/starkware-libs/starknet-specs/blob/0e859ff905795f789f1dfd6f7340cdaf5015acc8/api/starknet_write_api.json#L227
#[derive(thiserror::Error, Clone, Copy, Debug)]
pub enum StarknetRpcApiError {
    #[error("Failed to write transaction")]
    FailedToReceiveTxn = 1,
    #[error("Contract not found")]
    ContractNotFound = 20,
    #[error("Block not found")]
    BlockNotFound = 24,
    #[error("Invalid transaction index in a block")]
    InvalidTxnIndex = 27,
    #[error("Class hash not found")]
    ClassHashNotFound = 28,
    #[error("Transaction hash not found")]
    TxnHashNotFound = 29,
    #[error("Requested page size is too big")]
    PageSizeTooBig = 31,
    #[error("There are no blocks")]
    NoBlocks = 32,
    #[error("The supplied continuation token is invalid or unknown")]
    InvalidContinuationToken = 33,
    #[error("Too many keys provided in a filter")]
    TooManyKeysInFilter = 34,
    #[error("Failed to fetch pending transactions")]
    FailedToFetchPendingTransactions = 38,
    #[error("Contract error")]
    ContractError = 40,
    #[error("Invalid contract class")]
    InvalidContractClass = 50,
    #[error("Class already declared")]
    ClassAlreadyDeclared = 51,
    #[error("Account validation failed")]
    ValidationFailure = 55,
    #[error("The transaction version is not supported")]
    UnsupportedTxVersion = 61,
    #[error("Internal server error")]
    InternalServerError = 500,
    #[error("Unimplemented method")]
    UnimplementedMethod = 501,
    #[error("Too many storage keys requested")]
    ProofLimitExceeded = 10000,
}

impl From<mp_simulations::Error> for StarknetRpcApiError {
    fn from(value: mp_simulations::Error) -> Self {
        match value {
            mp_simulations::Error::ContractNotFound(_) => StarknetRpcApiError::ContractNotFound,
            mp_simulations::Error::TransactionExecutionFailed => StarknetRpcApiError::ContractError,
            mp_simulations::Error::MissingL1GasUsage => StarknetRpcApiError::InternalServerError,
            mp_simulations::Error::FailedToCreateATransactionalStorageExecution => {
                StarknetRpcApiError::InternalServerError
            }
        }
    }
}

impl From<StarknetRpcApiError> for jsonrpsee::core::Error {
    fn from(err: StarknetRpcApiError) -> Self {
        jsonrpsee::core::Error::Call(CallError::Custom(ErrorObject::owned(err as i32, err.to_string(), None::<()>)))
    }
}
