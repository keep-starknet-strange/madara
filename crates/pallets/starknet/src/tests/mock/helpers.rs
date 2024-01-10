use alloc::sync::Arc;

use mp_felt::Felt252Wrapper;
use mp_transactions::DeployAccountTransaction;
use sp_core::H256;
use starknet_api::api_core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::Calldata;
use starknet_core::utils::get_storage_var_address;
use starknet_crypto::FieldElement;

use crate::tests::constants::*;
use crate::types::ContractStorageKey;

/// Returns the storage key for a given storage name, keys and offset.
/// Calculates pedersen(sn_keccak(storage_name), keys) + storage_key_offset which is the key in the
/// starknet contract for storage_name(key_1, key_2, ..., key_n).
/// https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/#storage_variables
pub fn get_storage_key(
    address: &ContractAddress,
    storage_name: &str,
    keys: &[FieldElement],
    storage_key_offset: u64,
) -> ContractStorageKey {
    let storage_key_offset = H256::from_low_u64_be(storage_key_offset);
    let mut storage_key = get_storage_var_address(storage_name, keys).unwrap();
    storage_key += FieldElement::from_bytes_be(&storage_key_offset.to_fixed_bytes()).unwrap();
    (*address, StorageKey(PatriciaKey(Felt252Wrapper::from(storage_key).into())))
}

#[derive(Copy, Clone)]
pub enum AccountType {
    V0(AccountTypeV0Inner),
    V1(AccountTypeV1Inner),
}

#[derive(Copy, Clone)]
pub enum AccountTypeV0Inner {
    Argent,
    Openzeppelin,
    Braavos,
    BraavosProxy,
    NoValidate,
    InnerCall,
}

#[derive(Copy, Clone)]
pub enum AccountTypeV1Inner {
    NoValidate,
}

/// Returns the account address, class hash and calldata given an account type and given deploy salt
pub fn account_helper(account_type: AccountType) -> (ClassHash, Calldata) {
    let account_class_hash = get_account_class_hash(account_type);
    let calldata = get_account_calldata(account_type);
    let calldata = Calldata(Arc::new(calldata.into_iter().map(|s| StarkFelt::try_from(s).unwrap()).collect()));

    (account_class_hash, calldata)
}

/// Returns the class hash of a given account type
pub fn get_account_class_hash(account_type: AccountType) -> ClassHash {
    let class_hash = match account_type {
        AccountType::V0(inner) => match inner {
            AccountTypeV0Inner::Argent => ARGENT_ACCOUNT_CLASS_HASH_CAIRO_0,
            AccountTypeV0Inner::Braavos => BRAAVOS_ACCOUNT_CLASS_HASH_CAIRO_0,
            AccountTypeV0Inner::BraavosProxy => BRAAVOS_PROXY_CLASS_HASH_CAIRO_0,
            AccountTypeV0Inner::Openzeppelin => OPENZEPPELIN_ACCOUNT_CLASS_HASH_CAIRO_0,
            AccountTypeV0Inner::NoValidate => NO_VALIDATE_ACCOUNT_CLASS_HASH_CAIRO_0,
            AccountTypeV0Inner::InnerCall => UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH_CAIRO_0,
        },
        AccountType::V1(inner) => match inner {
            AccountTypeV1Inner::NoValidate => NO_VALIDATE_ACCOUNT_CLASS_HASH_CAIRO_1,
        },
    };
    ClassHash(StarkFelt::try_from(class_hash).unwrap())
}

/// Returns the required calldata for deploying the given account type
pub fn get_account_calldata(account_type: AccountType) -> Vec<&'static str> {
    match account_type {
        AccountType::V0(inner) => match inner {
            AccountTypeV0Inner::BraavosProxy => vec![
                BRAAVOS_ACCOUNT_CLASS_HASH_CAIRO_0, // Braavos account class hash
                "0x02dd76e7ad84dbed81c314ffe5e7a7cacfb8f4836f01af4e913f275f89a3de1a", // 'initializer' selector
            ],
            AccountTypeV0Inner::Openzeppelin => vec![ACCOUNT_PUBLIC_KEY],
            _ => vec![],
        },
        _ => vec![],
    }
}

/// Returns the account address for an account type
pub fn get_account_address(salt: Option<Felt252Wrapper>, account_type: AccountType) -> ContractAddress {
    let class_hash: Felt252Wrapper = get_account_class_hash(account_type).into();
    let calldata: Vec<_> =
        get_account_calldata(account_type).into_iter().map(|v| FieldElement::from_hex_be(v).unwrap()).collect();
    let contract_address_salt = salt.unwrap_or(*TEST_ACCOUNT_SALT);

    Felt252Wrapper(DeployAccountTransaction::calculate_contract_address(
        contract_address_salt.0,
        class_hash.0,
        &calldata,
    ))
    .into()
}
