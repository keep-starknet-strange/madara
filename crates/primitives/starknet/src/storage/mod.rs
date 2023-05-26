//! StarkNet storage primitives.

use scale_codec::{Decode, Encode};

/// Current version of pallet Starknet's storage schema is stored under this key.
pub const PALLET_STARKNET_SCHEMA: &[u8] = b":starknet_schema";

/// Pallet Starknet storage items.
/// Pallet name.
pub const PALLET_STARKNET: &[u8] = b"Starknet";
/// Starknet current block storage item.
pub const STARKNET_CURRENT_BLOCK: &[u8] = b"CurrentBlock";
/// Starknet contract class hash storage item.
pub const STARKNET_CONTRACT_CLASS_HASH: &[u8] = b"ContractClassHashes";
/// Starknet contract class storage item.
pub const STARKNET_CONTRACT_CLASS: &[u8] = b"ContractClasses";
/// Starknet nonce storage item.
pub const STARKNET_NONCE: &[u8] = b"Nonces";

/// The schema version for Pallet Starknet's storage.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum StarknetStorageSchemaVersion {
    /// Undefined schema.
    Undefined,
    /// Schema V1.
    V1,
}

impl Default for StarknetStorageSchemaVersion {
    fn default() -> Self {
        Self::Undefined
    }
}
