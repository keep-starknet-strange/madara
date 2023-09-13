//! Starknet primitives.

#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

// Include modules.
/// Starknet transaction constants.
pub mod constants;

/// Starknet block related functionality.
pub mod block;

/// Starknet transaction related functionality.
pub mod transaction;

/// Starknet crypto related functionality.
pub mod crypto;

/// Starknet storage primitives.
pub mod storage;

/// Starknet state related functionality.
pub mod state;

/// Starknet primitives traits.
pub mod traits;

/// Starknet Execution related functionality.
pub mod execution;

/// Starknet Fees related functionality.
pub mod fees;

/// Sequencer address inherent data.
pub mod sequencer_address;

/// Tests.
#[cfg(test)]
pub mod tests;
