use mp_felt::Felt252WrapperError;

#[warn(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub enum L1MessagesWorkerError {
    ConvertTransactionRuntimeApiError,
    MessageAlreadyProcessed,
    OffchainStorageError,
    RuntimeApiError,
    SubmitTxError,
    ToFeeError,
    ToTransactionError,
}

impl From<Felt252WrapperError> for L1MessagesWorkerError {
    fn from(_e: Felt252WrapperError) -> Self {
        L1MessagesWorkerError::ToTransactionError
    }
}
