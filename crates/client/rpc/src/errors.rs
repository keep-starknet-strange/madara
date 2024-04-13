use std::fmt::Display;

use blockifier::transaction::errors::TransactionExecutionError;
use jsonrpsee::types::error::{CallError, ErrorObject};
use log::error;
use mp_simulations::PlaceHolderErrorTypeForFailedStarknetExecution;
use sp_runtime::DispatchError;

// Comes from the RPC Spec:
// https://github.com/starkware-libs/starknet-specs/blob/0e859ff905795f789f1dfd6f7340cdaf5015acc8/api/starknet_write_api.json#L227
#[derive(thiserror::Error, Debug)]
pub enum StarknetRpcApiError {
    #[error("Failed to write transaction")]
    FailedToReceiveTxn,
    #[error("Contract not found")]
    ContractNotFound,
    #[error("Block not found")]
    BlockNotFound,
    #[error("Invalid transaction index in a block")]
    InvalidTxnIndex,
    #[error("Class hash not found")]
    ClassHashNotFound,
    #[error("Transaction hash not found")]
    TxnHashNotFound,
    #[error("Requested page size is too big")]
    PageSizeTooBig,
    #[error("There are no blocks")]
    NoBlocks,
    #[error("The supplied continuation token is invalid or unknown")]
    InvalidContinuationToken,
    #[error("Too many keys provided in a filter")]
    TooManyKeysInFilter,
    #[error("Failed to fetch pending transactions")]
    FailedToFetchPendingTransactions,
    #[error("Contract error: {0}")]
    ContractError(ContractErrorWrapper),
    #[error("Invalid contract class")]
    InvalidContractClass,
    #[error("Class already declared")]
    ClassAlreadyDeclared,
    #[error("Account validation failed")]
    ValidationFailure,
    #[error("The transaction version is not supported")]
    UnsupportedTxVersion,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Unimplemented method")]
    UnimplementedMethod,
    #[error("Too many storage keys requested")]
    ProofLimitExceeded,
}

impl From<StarknetRpcApiError> for jsonrpsee::core::Error {
    fn from(err: StarknetRpcApiError) -> Self {
        let code = match err {
            StarknetRpcApiError::FailedToReceiveTxn => 1,
            StarknetRpcApiError::ContractNotFound => 20,
            StarknetRpcApiError::BlockNotFound => 24,
            StarknetRpcApiError::InvalidTxnIndex => 27,
            StarknetRpcApiError::ClassHashNotFound => 28,
            StarknetRpcApiError::TxnHashNotFound => 29,
            StarknetRpcApiError::PageSizeTooBig => 31,
            StarknetRpcApiError::NoBlocks => 32,
            StarknetRpcApiError::InvalidContinuationToken => 33,
            StarknetRpcApiError::TooManyKeysInFilter => 34,
            StarknetRpcApiError::FailedToFetchPendingTransactions => 38,
            StarknetRpcApiError::ContractError(_) => 40,
            StarknetRpcApiError::InvalidContractClass => 50,
            StarknetRpcApiError::ClassAlreadyDeclared => 51,
            StarknetRpcApiError::ValidationFailure => 55,
            StarknetRpcApiError::UnsupportedTxVersion => 61,
            StarknetRpcApiError::InternalServerError => 500,
            StarknetRpcApiError::UnimplementedMethod => 501,
            StarknetRpcApiError::ProofLimitExceeded => 10000,
        };

        jsonrpsee::core::Error::Call(CallError::Custom(ErrorObject::owned(code, err.to_string(), None::<()>)))
    }
}

impl From<TransactionExecutionError> for StarknetRpcApiError {
    fn from(value: TransactionExecutionError) -> Self {
        StarknetRpcApiError::ContractError(ContractErrorWrapper::TransactionExecutionError(value))
    }
}

impl From<DispatchError> for ContractErrorWrapper {
    fn from(value: DispatchError) -> Self {
        ContractErrorWrapper::DispatchError(value)
    }
}

impl From<PlaceHolderErrorTypeForFailedStarknetExecution> for ContractErrorWrapper {
    fn from(value: PlaceHolderErrorTypeForFailedStarknetExecution) -> Self {
        ContractErrorWrapper::PlaceHolderErrorTypeForFailedStarknetExecution(value)
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
            ContractErrorWrapper::TransactionExecutionError(e) => write!(f, "{}", e),
            ContractErrorWrapper::PlaceHolderErrorTypeForFailedStarknetExecution(e) => write!(f, "{:?}", e),
        }
    }
}
