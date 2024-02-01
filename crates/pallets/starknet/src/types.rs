//! Starknet pallet custom types.
use blockifier::execution::contract_class::ContractClass;
use mp_felt::Felt252Wrapper;
use sp_core::ConstU32;
use sp_std::vec::Vec;
use starknet_api::api_core::{ClassHash, ContractAddress};
use starknet_api::state::StorageKey;
use starknet_api::stdlib::collections::HashMap;
use starknet_api::transaction::{Event, Fee, MessageToL1, TransactionHash};

/// Contract Storage Key
pub type ContractStorageKey = (ContractAddress, StorageKey);

/// Make this configurable. Max transaction/block
pub type MaxTransactionsPendingBlock = ConstU32<1073741824>;

pub type ContractClassMapping = HashMap<ClassHash, ContractClass>;

/// Type wrapper for a storage slot.
pub type StorageSlot = (StorageKey, Felt252Wrapper);

pub type CasmClassHash = ClassHash;
pub type SierraClassHash = ClassHash;
pub type SierraOrCasmClassHash = ClassHash;

/// Declare Transaction Output
#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeployAccountTransactionOutput {
    /// Transaction hash
    pub transaction_hash: Felt252Wrapper,
    /// Contract Address
    pub contract_address: ContractAddress,
}

/// Build invoke transaction for transfer utils
pub struct BuildTransferInvokeTransaction {
    pub sender_address: Felt252Wrapper,
    pub token_address: Felt252Wrapper,
    pub recipient: Felt252Wrapper,
    pub amount_low: Felt252Wrapper,
    pub amount_high: Felt252Wrapper,
    pub nonce: Felt252Wrapper,
}

#[derive(Clone, Debug, PartialEq, Eq, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionOutput {
    pub transaction_hash: TransactionHash,
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
}
