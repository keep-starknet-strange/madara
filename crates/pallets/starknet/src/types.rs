//! Starknet pallet custom types.
use blockifier::execution::contract_class::ContractClass;
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use sp_core::ConstU32;
use starknet_api::api_core::ClassHash;
use starknet_api::stdlib::collections::HashMap;

/// Nonce of a Starknet transaction.
pub type NonceWrapper = Felt252Wrapper;
/// Storage Key
pub type StorageKeyWrapper = Felt252Wrapper;
/// Contract Storage Key
pub type ContractStorageKeyWrapper = (ContractAddressWrapper, StorageKeyWrapper);

/// Make this configurable. Max transaction/block
pub type MaxTransactionsPendingBlock = ConstU32<1073741824>;

pub type ContractClassMapping = HashMap<ClassHash, ContractClass>;

/// Declare Transaction Output
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeployAccountTransactionOutput {
    /// Transaction hash
    pub transaction_hash: Felt252Wrapper,
    /// Contract Address
    pub contract_address: ContractAddressWrapper,
}
