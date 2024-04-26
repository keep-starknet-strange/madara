//! Starknet pallet custom types.
use std::collections::HashMap;

use blockifier::execution::contract_class::ContractClass;
use mp_felt::Felt252Wrapper;
use sp_core::ConstU32;
use sp_std::vec::Vec;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::hash::StarkHash;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Event, Fee, MessageToL1, TransactionHash};

/// Contract Storage Key
pub type ContractStorageKey = (ContractAddress, StorageKey);

/// Make this configurable. Max transaction/block
pub type MaxTransactionsPendingBlock = ConstU32<1073741824>;

pub type ContractClassMapping = HashMap<ClassHash, ContractClass>;

/// Type wrapper for a storage slot.
pub type StorageSlot = (StorageKey, Felt252Wrapper);

pub type CasmClassHash = StarkHash;
pub type SierraClassHash = StarkHash;
pub type SierraOrCasmClassHash = StarkHash;

/// Declare Transaction Output
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeployAccountTransactionOutput {
    /// Transaction hash
    pub transaction_hash: Felt252Wrapper,
    /// Contract Address
    pub contract_address: ContractAddress,
}

#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionOutput {
    pub transaction_hash: TransactionHash,
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
}
