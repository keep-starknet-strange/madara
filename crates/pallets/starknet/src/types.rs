//! Starknet pallet custom types.
use blockifier::execution::contract_class::ContractClass;
use mp_starknet::execution::ContractAddressWrapper;
use sp_core::{ConstU32, H256, U256};
use starknet_api::api_core::ClassHash;
use starknet_api::stdlib::collections::HashMap;

/// A field element in the Starknet protocol.
pub type FieldElement = [u8; 32];

/// Nonce of a Starknet transaction.
pub type NonceWrapper = U256;
/// Storage Key
pub type StorageKeyWrapper = H256;
/// Contract Storage Key
pub type ContractStorageKeyWrapper = (ContractAddressWrapper, StorageKeyWrapper);
/// Felt
pub type StarkFeltWrapper = U256;

/// Make this configurable. Max transaction/block
pub type MaxTransactionsPendingBlock = ConstU32<1073741824>;

pub type ContractClassMapping = HashMap<ClassHash, ContractClass>;
