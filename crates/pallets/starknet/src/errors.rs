use blockifier::execution::errors::{EntryPointExecutionError, PreExecutionError};
use blockifier::transaction::errors::TransactionExecutionError;
pub use pallet::*;
use frame_system::pallet_prelude::*;

/// Wrapper Type For Blockifier Errors
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum BlockifierErrors {
    EntryPointExecutionError(EntryPointExecutionError),
    PreExecutionError(PreExecutionError),
    TransactionExecutionError(TransactionExecutionError),
}
