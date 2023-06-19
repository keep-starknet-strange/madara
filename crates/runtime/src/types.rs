//! Common types used in the runtime.
//! This file is the canonical source of truth for the types used in the runtime.

use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiSignature;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// The maximum amount of steps allowed for an invoke transaction. (?)
pub type InvokeTxMaxNSteps = u32;
/// The maximum amount of steps allowed for validation. (?)
pub type ValidateMaxNSteps = u32;
