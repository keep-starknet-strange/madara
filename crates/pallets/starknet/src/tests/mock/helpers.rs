use mp_starknet::execution::types::Felt252Wrapper;
use sp_core::H256;
use starknet_api::api_core::{calculate_contract_address as _calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_api::StarknetApiError;
use starknet_core::utils::get_storage_var_address;
use starknet_crypto::FieldElement;

use crate::tests::constants::*;
use crate::types::ContractStorageKeyWrapper;

/// Returns the storage key for a given storage name, keys and offset.
/// Calculates pedersen(sn_keccak(storage_name), keys) + storage_key_offset which is the key in the
/// starknet contract for storage_name(key_1, key_2, ..., key_n).
/// https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/#storage_variables
pub fn get_storage_key(
    address: &Felt252Wrapper,
    storage_name: &str,
    keys: &[Felt252Wrapper],
    storage_key_offset: u64,
) -> ContractStorageKeyWrapper {
    let storage_key_offset = H256::from_low_u64_be(storage_key_offset);
    let mut storage_key = get_storage_var_address(
        storage_name,
        keys.iter().map(|x| FieldElement::from(*x)).collect::<Vec<_>>().as_slice(),
    )
    .unwrap();
    storage_key += FieldElement::from_bytes_be(&storage_key_offset.to_fixed_bytes()).unwrap();
    (*address, storage_key.into())
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
pub fn account_helper(
    salt: Felt252Wrapper,
    account_type: AccountType,
) -> (Felt252Wrapper, Felt252Wrapper, Vec<&'static str>) {
    let account_class_hash = get_account_class_hash(account_type);
    let calldata = get_account_calldata(account_type);
    let addr = calculate_contract_address(salt, account_class_hash, calldata.clone()).unwrap();
    (addr.0.0.into(), account_class_hash, calldata)
}

/// Returns the class hash of a given account type
pub fn get_account_class_hash(account_type: AccountType) -> Felt252Wrapper {
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
    FieldElement::from_hex_be(class_hash).unwrap().into()
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
pub fn get_account_address(account_type: AccountType) -> Felt252Wrapper {
    account_helper(*TEST_ACCOUNT_SALT, account_type).0
}

/// Calculate the address of a contract.
/// # Arguments
/// * `salt` - The salt of the contract.
/// * `class_hash` - The hash of the contract class.
/// * `constructor_calldata` - The calldata of the constructor.
/// # Returns
/// The address of the contract.
/// # Errors
/// If the contract address cannot be calculated.
pub fn calculate_contract_address(
    salt: Felt252Wrapper,
    class_hash: Felt252Wrapper,
    constructor_calldata: Vec<&str>,
) -> Result<ContractAddress, StarknetApiError> {
    _calculate_contract_address(
        ContractAddressSalt(StarkFelt::new(salt.0.to_bytes_be())?),
        ClassHash(StarkFelt::new(class_hash.0.to_bytes_be())?),
        &Calldata(
            constructor_calldata
                .clone()
                .into_iter()
                .map(|x| StarkFelt::try_from(x).unwrap())
                .collect::<Vec<StarkFelt>>()
                .into(),
        ),
        ContractAddress::default(),
    )
}
