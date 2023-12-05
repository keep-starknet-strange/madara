//! Starknet storage primitives.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec::Vec;

use lazy_static::lazy_static;
use sp_io::hashing::twox_128;

/// Current version of pallet Starknet's storage schema is stored under this key.
pub const PALLET_STARKNET_SCHEMA: &[u8] = b":starknet_schema";

/// System storage items.
/// Pallet name.
pub const PALLET_SYSTEM: &[u8] = b"System";
/// System events storage item.
pub const SYSTEM_EVENTS: &[u8] = b"Events";

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
/// Starknet storage
pub const STARKNET_STORAGE: &[u8] = b"StorageView";
/// Compiled class hashes
pub const STARKNET_COMPILED_CLASS_HASH: &[u8] = b"CompiledClassHashes";

lazy_static! {
    pub static ref SN_NONCE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_NONCE)].concat();
    pub static ref SN_CONTRACT_CLASS_HASH_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS_HASH)].concat();
    pub static ref SN_CONTRACT_CLASS_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS)].concat();
    pub static ref SN_STORAGE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_STORAGE)].concat();
    pub static ref SN_COMPILED_CLASS_HASH_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_COMPILED_CLASS_HASH)].concat();
}

/// The schema version for Pallet Starknet's storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Decode, parity_scale_codec::Encode))]
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
