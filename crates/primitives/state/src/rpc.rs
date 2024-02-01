//! Starknet rpc state primitives.

use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;
#[cfg(feature = "serde")]
use mp_felt::UfeHex;

/// Replaced class.
///
/// The list of contracts whose class was replaced.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct ReplacedClassItem {
    /// The address of the contract whose class was replaced
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub contract_address: Felt252Wrapper,
    /// The new class hash
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub class_hash: Felt252Wrapper,
}

/// Deployed contract item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct DeployedContractItem {
    /// The address of the contract
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub address: Felt252Wrapper,
    /// The hash of the contract code
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub class_hash: Felt252Wrapper,
}

/// New classes.
///
/// The declared class hash and compiled class hash.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct DeclaredClassItem {
    /// The hash of the declared class
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub class_hash: Felt252Wrapper,
    /// The cairo assembly hash corresponding to the declared class
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub compiled_class_hash: Felt252Wrapper,
}

/// Contract storage diff item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct ContractStorageDiffItem {
    /// The contract address for which the storage changed
    pub address: Felt252Wrapper,
    /// The changes in the storage of the contract
    pub storage_entries: Vec<StorageEntry>,
}

/// Storage diff item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct StorageEntry {
    /// The key of the changed value
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub key: Felt252Wrapper,
    /// The new value applied to the given address
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub value: Felt252Wrapper,
}

/// Nonce update.
///
/// The updated nonce per contract address.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct NonceUpdate {
    /// The address of the contract
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub contract_address: Felt252Wrapper,
    /// The nonce for the given address at the end of the block
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub nonce: Felt252Wrapper,
}

/// The change in state applied in this block, given as a mapping of addresses to the new values
/// and/or new contracts.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde_with::serde_as)]
pub struct StateDiff {
    /// Storage diffs
    pub storage_diffs: Vec<ContractStorageDiffItem>,
    /// Deprecated declared classes
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
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
