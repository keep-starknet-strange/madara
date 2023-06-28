//! Starknet execution functionality.

use alloc::collections::BTreeMap;

use frame_support::BoundedBTreeMap;
use serde::de::Error as DeserializationError;
use serde::{Deserialize, Deserializer, Serializer};
use sp_core::Get;

/// Call Entrypoint Wrapper related types
pub mod call_entrypoint_wrapper;
/// Entrypoint Wrapper related types
pub mod entrypoint_wrapper;
/// Felt252Wrapper type
pub mod felt252_wrapper;

/// All the types related to the execution of a transaction.
pub mod types {
    /// Type wrapper for a contract address.
    pub type ContractAddressWrapper = Felt252Wrapper;

    /// Type wrapper for a storage key;
    pub type StorageKeyWrapper = Felt252Wrapper;

    /// Wrapper type for class hash field.
    pub type ClassHashWrapper = Felt252Wrapper;

    pub use super::call_entrypoint_wrapper::*;
    pub use super::entrypoint_wrapper::*;
    pub use super::felt252_wrapper::*;
}

#[cfg(feature = "std")]
mod reexport_private_types {
    use frame_support::Serialize;

    use super::*;

    /// Serialization of [BoundedBTreeMap].
    /// This is needed for the genesis config.
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

    /// Serialization of [`Option<BoundedBTreeMap>`].
    /// This is needed for the genesis config.
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

    /// Deserialization of an [`Option<BoundedBTreeMap>`] object.
    /// This is needed for the genesis config.
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
            Some(btree_map) => Some(BoundedBTreeMap::try_from(btree_map).map_err(|_| {
                DeserializationError::custom("Couldn't convert BTreeMap to BoundedBTreeMap".to_string())
            })?),
            None => None,
        })
    }
}

#[cfg(feature = "std")]
pub use reexport_private_types::*;
