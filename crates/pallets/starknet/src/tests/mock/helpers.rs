use core::str::FromStr;

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
    Argent,
    Openzeppelin,
    Braavos,
    BraavosProxy,
    NoValidate,
    InnerCall,
}

/// Returns the account address, class hash and calldata given an account type and given deploy salt
pub fn account_helper(salt: &str, account_type: AccountType) -> (Felt252Wrapper, Felt252Wrapper, Vec<&str>) {
    let account_class_hash = get_account_class_hash(account_type);
    let calldata = get_account_calldata(account_type);
    let account_salt = H256::from_str(salt).unwrap();
    let addr = calculate_contract_address(account_salt, account_class_hash.into(), calldata.clone()).unwrap();
    (addr.0.0.into(), account_class_hash, calldata)
}

/// Returns the class hash of a given account type
pub fn get_account_class_hash(account_type: AccountType) -> Felt252Wrapper {
    let class_hash = match account_type {
        AccountType::Argent => ARGENT_ACCOUNT_CLASS_HASH,
        AccountType::Braavos => BRAAVOS_ACCOUNT_CLASS_HASH,
        AccountType::BraavosProxy => BRAAVOS_PROXY_CLASS_HASH,
        AccountType::Openzeppelin => OPENZEPPELIN_ACCOUNT_CLASS_HASH,
        AccountType::NoValidate => NO_VALIDATE_ACCOUNT_CLASS_HASH,
        AccountType::InnerCall => UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH,
    };
    FieldElement::from_hex_be(class_hash).unwrap().into()
}

/// Returns the required calldata for deploying the given account type
pub fn get_account_calldata(account_type: AccountType) -> Vec<&'static str> {
    match account_type {
        AccountType::BraavosProxy => vec![
            BRAAVOS_ACCOUNT_CLASS_HASH, // Braavos account class hash
            "0x02dd76e7ad84dbed81c314ffe5e7a7cacfb8f4836f01af4e913f275f89a3de1a", // 'initializer' selector
        ],
        AccountType::Openzeppelin => vec![ACCOUNT_PUBLIC_KEY],
        _ => vec![],
    }
}

/// Returns the account address for an account type
pub fn get_account_address(account_type: AccountType) -> Felt252Wrapper {
    account_helper(TEST_ACCOUNT_SALT, account_type).0
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
    salt: H256,
    class_hash: H256,
    constructor_calldata: Vec<&str>,
) -> Result<ContractAddress, StarknetApiError> {
    _calculate_contract_address(
        ContractAddressSalt(StarkFelt::new(salt.0)?),
        ClassHash(StarkFelt::new(class_hash.0)?),
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
