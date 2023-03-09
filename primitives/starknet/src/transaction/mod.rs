//! Starknet transaction related functionality.

use sp_core::U256;

/// Representation of a Starknet transaction.
#[derive(
    Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, Default, codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    /// The version of the transaction.
    pub version: U256,
}
