use core::fmt;

use blockifier::transaction::errors::TransactionExecutionError;
use jsonrpsee::types::error::{CallError, ErrorObject};
use mp_simulations::PlaceHolderErrorTypeForFailedStarknetExecution;
use starknet_api::api_core::ContractAddress;
use thiserror::Error;

// Comes from the RPC Spec:
// https://github.com/starkware-libs/starknet-specs/blob/0e859ff905795f789f1dfd6f7340cdaf5015acc8/api/starknet_write_api.json#L227
#[derive(Error, Debug)]
pub enum StarknetRpcApiError {
    #[error("Failed to write transaction")]
    FailedToReceiveTxn,
    #[error("Contract not found: {0}")]
    ContractNotFound(DisplayableContractAddress),
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
#[error("Contract Error")]
pub struct ContractError {
    revert_error: String,
}

#[derive(Debug)]
pub struct DisplayableContractAddress(pub ContractAddress);

impl fmt::Display for DisplayableContractAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // unwrapping all the layers of different types to get display from starkhash
        let address = &self.0.0.0;
        write!(f, "{}", address)
    }
}

impl From<ContractAddress> for DisplayableContractAddress {
    fn from(value: ContractAddress) -> Self {
        DisplayableContractAddress(value)
    }
}

impl From<StarknetRpcApiError> for jsonrpsee::core::Error {
    fn from(err: StarknetRpcApiError) -> Self {
        let code = match err {
            StarknetRpcApiError::FailedToReceiveTxn => 1,
            StarknetRpcApiError::ContractNotFound(_) => 20,
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

impl From<PlaceHolderErrorTypeForFailedStarknetExecution> for ContractError {
    fn from(e: PlaceHolderErrorTypeForFailedStarknetExecution) -> Self {
        let format = format!("{:?}", e);
        ContractError { revert_error: format }
    }
}

impl From<TransactionExecutionError> for StarknetRpcApiError {
    fn from(e: TransactionExecutionError) -> Self {
        StarknetRpcApiError::ContractError(e.into())
    }
}

impl From<PlaceHolderErrorTypeForFailedStarknetExecution> for StarknetRpcApiError {
    fn from(e: PlaceHolderErrorTypeForFailedStarknetExecution) -> Self {
        StarknetRpcApiError::ContractError(e.into())
    }
}

impl From<mp_simulations::Error> for StarknetRpcApiError {
    fn from(value: mp_simulations::Error) -> Self {
        match value {
            mp_simulations::Error::ContractNotFound(address) => StarknetRpcApiError::ContractNotFound(address.into()),
            mp_simulations::Error::TransactionExecutionFailed(e) => StarknetRpcApiError::ContractError(e.into()),
            mp_simulations::Error::MissingL1GasUsage => StarknetRpcApiError::InternalServerError,
            mp_simulations::Error::FailedToCreateATransactionalStorageExecution => {
                StarknetRpcApiError::InternalServerError
            }
        }
    }
}
