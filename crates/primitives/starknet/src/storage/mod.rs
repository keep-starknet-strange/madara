//! StarkNet storage primitives.

use codec::{Decode, Encode};

/// Current version of pallet Starknet's storage schema is stored under this key.
pub const PALLET_STARKNET_SCHEMA: &[u8] = b":starknet_schema";
/// Cached version of pallet Starknet's storage schema is stored under this key in the AuxStore.
pub const PALLET_STARKNET_SCHEMA_CACHE: &[u8] = b":starknet_schema_cache";

/// Pallet Starknet storage items.
/// Pallet name.
pub const PALLET_STARKNET: &[u8] = b"Starknet";
/// Starknet current block storage item.
pub const STARKNET_CURRENT_BLOCK: &[u8] = b"CurrentBlock";

/// The schema version for Pallet Starknet's storage.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum StarknetStorageSchema {
    /// Undefined schema.
    Undefined,
    /// Schema V1.
    V1,
}

impl Default for StarknetStorageSchema {
    fn default() -> Self {
        Self::Undefined
    }
}
