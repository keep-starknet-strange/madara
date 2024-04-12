use std::fmt::Display;

use blockifier::transaction::errors::TransactionExecutionError;
use jsonrpsee::types::error::{CallError, ErrorObject};
use mp_simulations::PlaceHolderErrorTypeForFailedStarknetExecution;
use sp_runtime::DispatchError;

// Comes from the RPC Spec:
// https://github.com/starkware-libs/starknet-specs/blob/0e859ff905795f789f1dfd6f7340cdaf5015acc8/api/starknet_write_api.json#L227
#[repr(i32)]
#[derive(thiserror::Error, Debug)]
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
    #[error("Contract error: {0}")]
    ContractError(ContractErrorWrapper) = 40,
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

impl From<StarknetRpcApiError> for jsonrpsee::core::Error {
    fn from(err: StarknetRpcApiError) -> Self {
        jsonrpsee::core::Error::Call(CallError::Custom(ErrorObject::owned(40, err.to_string(), None::<()>)))
    }
}

impl From<TransactionExecutionError> for StarknetRpcApiError {
    fn from(value: TransactionExecutionError) -> Self {
        StarknetRpcApiError::ContractError(ContractErrorWrapper::TransactionExecutionError(value))
    }
}

impl From<DispatchError> for StarknetRpcApiError {
    fn from(value: DispatchError) -> Self {
        StarknetRpcApiError::ContractError(ContractErrorWrapper::DispatchError(value))
    }
}

#[derive(Debug)]
pub enum ContractErrorWrapper {
    DispatchError(DispatchError),
    TransactionExecutionError(TransactionExecutionError),
    PlaceHolderErrorTypeForFailedStarknetExecution(PlaceHolderErrorTypeForFailedStarknetExecution),
}

impl Display for ContractErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractErrorWrapper::DispatchError(e) => write!(f, "{:?}", e),
            ContractErrorWrapper::TransactionExecutionError(e) => write!(f, "{:?}", e),
            ContractErrorWrapper::PlaceHolderErrorTypeForFailedStarknetExecution(e) => write!(f, "{:?}", e),
        }
    }
}
