//! Starknet execution functionality.
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::format;
use core::num::ParseIntError;

use frame_support::BoundedBTreeMap;
#[cfg(feature = "std")]
use frame_support::Serialize;
use serde::de::Error as DeserializationError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;
use sp_core::Get;
mod call_entrypoint_wrapper;
mod contract_class_wrapper;
mod entrypoint_wrapper;
mod program_wrapper;

/// Serialization of [BoundedBTreeMap].
/// This is needed for the genesis config.
#[cfg(feature = "std")]
pub fn serialize_bounded_btreemap<SE: Serializer, K, V, S>(
    v: &BoundedBTreeMap<K, V, S>,
    serializer: SE,
) -> Result<SE::Ok, SE::Error>
where
    K: scale_codec::Decode + Ord + Serialize + Clone,
    V: scale_codec::Decode + Serialize + Clone,
    S: Get<u32>,
{
    v.clone().into_inner().serialize(serializer)
}

/// Serialization of [Option<BoundedBTreeMap>].
/// This is needed for the genesis config.
#[cfg(feature = "std")]
pub fn serialize_option_bounded_btreemap<SE: Serializer, K, V, S>(
    v: &Option<BoundedBTreeMap<K, V, S>>,
    serializer: SE,
) -> Result<SE::Ok, SE::Error>
where
    K: scale_codec::Decode + Ord + Serialize + Clone,
    V: scale_codec::Decode + Serialize + Clone,
    S: Get<u32>,
{
    v.clone().map(|val| val.into_inner()).serialize(serializer)
}

/// Deserialization of [BoundedBTreeMap].
/// This is needed for the genesis config.
#[cfg(feature = "std")]
pub fn deserialize_bounded_btreemap<'de, D: Deserializer<'de>, K, V, S>(
    deserializer: D,
) -> Result<BoundedBTreeMap<K, V, S>, D::Error>
where
    K: scale_codec::Decode + Ord + Deserialize<'de>,
    V: scale_codec::Decode + Deserialize<'de>,
    S: Get<u32>,
{
    let btree_map = BTreeMap::deserialize(deserializer)?;
    BoundedBTreeMap::try_from(btree_map)
        .map_err(|_| DeserializationError::custom("Couldn't convert BTreeMap to BoundedBTreeMap".to_string()))
}

/// Deserialization of an [Option<BoundedBTreeMap>] object.
/// This is needed for the genesis config.
#[cfg(feature = "std")]
pub fn deserialize_option_bounded_btreemap<'de, D: Deserializer<'de>, K, V, S>(
    deserializer: D,
) -> Result<Option<BoundedBTreeMap<K, V, S>>, D::Error>
where
    K: scale_codec::Decode + Ord + Deserialize<'de>,
    V: scale_codec::Decode + Deserialize<'de>,
    S: Get<u32>,
{
    let opt_btree_map = Option::<BTreeMap<K, V>>::deserialize(deserializer)?;
    Ok(match opt_btree_map {
        Some(btree_map) => {
            Some(BoundedBTreeMap::try_from(btree_map).map_err(|_| {
                DeserializationError::custom("Couldn't convert BTreeMap to BoundedBTreeMap".to_string())
            })?)
        }
        None => None,
    })
}

pub fn number_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    let u128_value = match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            number.as_u64().ok_or(DeserializationError::custom("Cannot cast number to u128."))? as u128
        }
        Value::String(s) => hex_string_try_into_u128(&s).map_err(DeserializationError::custom)?,
        _ => return Err(DeserializationError::custom("Cannot cast value into u128.")),
    };
    Ok(u128_value)
}
pub fn number_or_string_to_bytes<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
    let u128_value = match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            let mut bytes: [u8; 32] = [0u8; 32];
            bytes[25..].copy_from_slice(
                &number.as_u64().ok_or(DeserializationError::custom("Cannot cast number to u128."))?.to_be_bytes(),
            );
            bytes
        }
        Value::String(s) => {
            hex_string_to_bytes(&s).map_err(|_| DeserializationError::custom("Failed to parse hex to bytes"))?
        }
        _ => return Err(DeserializationError::custom("Cannot cast value into bytes.")),
    };
    Ok(u128_value)
}

fn hex_string_to_bytes(s: &str) -> Result<[u8; 32], ParseIntError> {
    let mut bytes = [0u8; 32];
    let s = s.trim_start_matches("0x");
    let s = if s.len() % 2 != 0 { format!("0{:}", s) } else { s.to_owned() };
    for (id, i) in (0..s.len()).step_by(2).enumerate() {
        bytes[id] = u8::from_str_radix(&s[i..i + 2], 16)?;
    }
    Ok(bytes)
}
fn hex_string_try_into_u128(hex_string: &str) -> Result<u128, ParseIntError> {
    u128::from_str_radix(hex_string.trim_start_matches("0x"), 16)
}

/// All the types related to the execution of a transaction.
pub mod types {
    /// Type wrapper for a contract address.
    pub type ContractAddressWrapper = [u8; 32];

    /// Wrapper type for class hash field.
    pub type ClassHashWrapper = [u8; 32];
    pub use super::call_entrypoint_wrapper::*;
    pub use super::contract_class_wrapper::*;
    pub use super::entrypoint_wrapper::*;
    pub use super::program_wrapper::*;
}
