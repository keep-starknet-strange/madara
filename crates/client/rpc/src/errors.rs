use blockifier::transaction::errors::TransactionExecutionError;
use jsonrpsee::types::error::{CallError, ErrorObject};
use thiserror::Error;

// Comes from the RPC Spec:
// https://github.com/starkware-libs/starknet-specs/blob/0e859ff905795f789f1dfd6f7340cdaf5015acc8/api/starknet_write_api.json#L227
#[derive(Error, Debug)]
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
    #[error(transparent)]
    ContractError(#[from] ContractError),
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

#[derive(Debug, Error)]
#[error("Contract SimulationError")]
pub struct ContractError {
    revert_error: String,
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

        let data = match &err {
            StarknetRpcApiError::ContractError(ref error) => Some(serde_json::json!({
                "revert_error": error.revert_error
            })),
            _ => None,
        };

        jsonrpsee::core::Error::Call(CallError::Custom(ErrorObject::owned(code, err.to_string(), data)))
    }
}

impl From<String> for ContractError {
    fn from(value: String) -> Self {
        ContractError { revert_error: value }
    }
}

impl From<TransactionExecutionError> for ContractError {
    fn from(e: TransactionExecutionError) -> Self {
        ContractError { revert_error: e.to_string() }
    }
}

impl From<TransactionExecutionError> for StarknetRpcApiError {
    fn from(e: TransactionExecutionError) -> Self {
        StarknetRpcApiError::ContractError(e.into())
    }
}

impl From<mp_simulations::SimulationError> for StarknetRpcApiError {
    fn from(value: mp_simulations::SimulationError) -> Self {
        match value {
            mp_simulations::SimulationError::ContractNotFound => StarknetRpcApiError::ContractNotFound,
            mp_simulations::SimulationError::TransactionExecutionFailed(e) => StarknetRpcApiError::ContractError(e.into()),
            mp_simulations::SimulationError::MissingL1GasUsage
            | mp_simulations::SimulationError::FailedToCreateATransactionalStorageExecution
            | mp_simulations::SimulationError::StateDiff => StarknetRpcApiError::InternalServerError,
        }
    }
}
