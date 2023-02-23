//! Starknet primitives.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub use codec;
#[doc(hidden)]
pub use scale_info;
#[cfg(feature = "std")]
#[doc(hidden)]
pub use serde;
#[doc(hidden)]
pub use sp_std;

// Include modules.
/// Starknet block related functionality.
pub mod block;
/// Starknet crypto related functionality.
pub mod crypto;

/// Serializations and deserializations.
pub mod starknet_serde;
