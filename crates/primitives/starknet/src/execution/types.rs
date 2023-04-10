use blockifier::execution::errors::EntryPointExecutionError;
use starknet_api::StarknetApiError;

/// Wrapper type for transaction execution result.
pub type EntryPointExecutionResultWrapper<T> = Result<T, EntryPointExecutionErrorWrapper>;

/// Wrapper type for transaction execution error.
#[derive(Debug)]
pub enum EntryPointExecutionErrorWrapper {
    /// Transaction execution error.
    EntryPointExecution(EntryPointExecutionError),
    /// Starknet API error.
    StarknetApi(StarknetApiError),
    /// Block context serialization error.
    BlockContextSerializationError,
}
