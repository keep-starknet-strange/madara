//! Starknet rpc state primitives.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::vec::Vec;

use mp_felt::{Felt252Wrapper, UfeHex};
use serde_with::serde_as;

/// Replaced class.
///
/// The list of contracts whose class was replaced.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReplacedClassItem {
    /// The address of the contract whose class was replaced
    #[serde_as(as = "UfeHex")]
    pub contract_address: Felt252Wrapper,
    /// The new class hash
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt252Wrapper,
}

/// Deployed contract item.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeployedContractItem {
    /// The address of the contract
    #[serde_as(as = "UfeHex")]
    pub address: Felt252Wrapper,
    /// The hash of the contract code
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt252Wrapper,
}

/// New classes.
///
/// The declared class hash and compiled class hash.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeclaredClassItem {
    /// The hash of the declared class
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt252Wrapper,
    /// The cairo assembly hash corresponding to the declared class
    #[serde_as(as = "UfeHex")]
    pub compiled_class_hash: Felt252Wrapper,
}

/// Contract storage diff item.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContractStorageDiffItem {
    /// The contract address for which the storage changed
    pub address: Felt252Wrapper,
    /// The changes in the storage of the contract
    pub storage_entries: Vec<StorageEntry>,
}

/// Storage diff item.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StorageEntry {
    /// The key of the changed value
    #[serde_as(as = "UfeHex")]
    pub key: Felt252Wrapper,
    /// The new value applied to the given address
    #[serde_as(as = "UfeHex")]
    pub value: Felt252Wrapper,
}

/// Nonce update.
///
/// The updated nonce per contract address.
#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NonceUpdate {
    /// The address of the contract
    #[serde_as(as = "UfeHex")]
    pub contract_address: Felt252Wrapper,
    /// The nonce for the given address at the end of the block
    #[serde_as(as = "UfeHex")]
    pub nonce: Felt252Wrapper,
}

/// The change in state applied in this block, given as a mapping of addresses to the new values
/// and/or new contracts.
#[serde_with::serde_as]
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateDiff {
    /// Storage diffs
    pub storage_diffs: Vec<ContractStorageDiffItem>,
    /// Deprecated declared classes
    #[serde_as(as = "Vec<UfeHex>")]
    pub deprecated_declared_classes: Vec<Felt252Wrapper>,
    /// Declared classes
    pub declared_classes: Vec<DeclaredClassItem>,
    /// Deployed contracts
    pub deployed_contracts: Vec<DeployedContractItem>,
    /// Replaced classes
    pub replaced_classes: Vec<ReplacedClassItem>,
    /// Nonces
    pub nonces: Vec<NonceUpdate>,
}
