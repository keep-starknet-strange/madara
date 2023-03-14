//! Starknet transaction related functionality.

use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};

type MaxSignatureFields = ConstU32<4294967295>;
/// Representation of a Starknet transaction.
#[derive(
    Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, Default, codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    /// The version of the transaction.
    pub version: U256,
    /// Transaction hash.
    pub hash: H256,
    /// Signature.
    pub signature: BoundedVec<H256, MaxSignatureFields>,
}

impl Transaction {
    /// Creates a new instance of a transaction.
    pub fn new(version: U256, hash: H256, signature: BoundedVec<H256, MaxSignatureFields>) -> Self {
        Self { version, hash, signature }
    }

    /// Creates a new instance of a transaction without signature.
    pub fn from_tx_hash(hash: H256) -> Self {
        Self { hash, ..Self::default() }
    }
}
