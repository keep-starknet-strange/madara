//! Starknet state primitives.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::vec::Vec;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{ContractStorageKey, StateChangesCount};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{StateReader, StateResult};
use mp_felt::{Felt252Wrapper, UfeHex};
use serde_with::serde_as;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::stdlib::collections::HashMap;

type ContractClassMapping = HashMap<ClassHash, ContractClass>;

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
#[derive(Debug, Clone)]
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

impl Default for StateDiff {
    fn default() -> Self {
        StateDiff {
            storage_diffs: Vec::default(),
            deprecated_declared_classes: Vec::default(),
            declared_classes: Vec::default(),
            deployed_contracts: Vec::default(),
            replaced_classes: Vec::default(),
            nonces: Vec::default(),
        }
    }
}

/// This trait allows to get the state changes of a starknet tx and therefore enables computing the
/// fees.
pub trait StateChanges {
    /// This function counts the storage var updates implied by a transaction and the newly declared
    /// class hashes.
    fn count_state_changes(&self) -> StateChangesCount;
}

/// A simple implementation of `StateReader` using `HashMap`s as storage.
#[derive(Debug, Default)]
pub struct DictStateReader {
    /// The storage layout.
    pub storage_view: HashMap<ContractStorageKey, StarkFelt>,
    /// The nonce of each contract.
    pub address_to_nonce: HashMap<ContractAddress, Nonce>,
    /// The class hash of each contract.
    pub address_to_class_hash: HashMap<ContractAddress, ClassHash>,
    /// The class of each class hash.
    pub class_hash_to_class: ContractClassMapping,
}

impl StateReader for DictStateReader {
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_storage_key = (contract_address, key);
        let value = self.storage_view.get(&contract_storage_key).copied().unwrap_or_default();
        Ok(value)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let nonce = self.address_to_nonce.get(&contract_address).copied().unwrap_or_default();
        Ok(nonce)
    }

    fn get_compiled_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<ContractClass> {
        let contract_class = self.class_hash_to_class.get(class_hash).cloned();
        match contract_class {
            Some(contract_class) => Ok(contract_class),
            None => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }

    fn get_compiled_class_hash(&mut self, _class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        // FIXME 708
        Ok(CompiledClassHash::default())
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let class_hash = self.address_to_class_hash.get(&contract_address).copied().unwrap_or_default();
        Ok(class_hash)
    }
}

#[cfg(test)]
mod tests;
