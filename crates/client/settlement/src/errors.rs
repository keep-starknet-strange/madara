use std::time::Duration;

use sp_runtime::traits::Block;
use starknet_api::hash::StarkHash;

use crate::{ethereum, RetryStrategy};

/// Settlement error type.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error<B: Block> {
    #[error("Blockchain error: {0}")]
    Blockchain(#[from] sp_blockchain::Error),

    #[error("Starknet API error: {0}")]
    StarknetApi(#[from] starknet_api::StarknetApiError),

    #[error("Failed to find Madara log: {0}")]
    DigestLog(#[from] mp_digest_log::FindLogError),

    #[error("Runtime API error: {0}")]
    RuntimeApi(#[from] sp_api::ApiError),

    #[error("Ethereum client error: {0}")]
    EthereumClient(#[from] ethereum::errors::Error),

    #[error("Failed to find Substrate block hash for Starknet block #{0}")]
    UnknownStarknetBlock(u64),

    #[error("Failed to find Substrate block header for hash: {0}")]
    UnknownSubstrateBlock(B::Hash),

    #[error("Unexpected global state root for block #{height}: expected {expected}, got {actual}")]
    StateRootMismatch { height: u64, expected: StarkHash, actual: StarkHash },

    #[error("Unexpected Starknet OS program hash: expected {expected}, got {actual}")]
    ProgramHashMismatch { expected: StarkHash, actual: StarkHash },

    #[error("Unexpected Starknet OS config hash: expected {expected}, got {actual}")]
    ConfigHashMismatch { expected: StarkHash, actual: StarkHash },

    #[error("Starknet state is not initialized yet")]
    StateNotInitialized,
}

pub type Result<T, B> = std::result::Result<T, Error<B>>;

pub struct RetryOnRecoverableErrors {
    pub delay: Duration,
}

impl<B: Block> RetryStrategy<B> for RetryOnRecoverableErrors {
    fn can_retry(&self, error: &Error<B>) -> Option<Duration> {
        match error {
            // List of non-recoverable errors
            Error::StateRootMismatch { .. } => None,
            Error::ConfigHashMismatch { .. } => None,
            Error::ProgramHashMismatch { .. } => None,
            // Otherwise we can continue after some delay
            _ => Some(self.delay),
        }
    }
}
