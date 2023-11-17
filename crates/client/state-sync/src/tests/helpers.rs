use alloc::sync::Arc;

use mp_felt::Felt252Wrapper;
use mp_transactions::DeployAccountTransaction;
use starknet_api::api_core::{ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
use starknet_crypto::FieldElement;

use super::constants::*;
pub extern crate alloc;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum AccountType {
    V0(AccountTypeV0Inner),
    V1(AccountTypeV1Inner),
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum AccountTypeV0Inner {
    Argent,
    Openzeppelin,
    Braavos,
    BraavosProxy,
    NoValidate,
    InnerCall,
}

#[allow(dead_code)]
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
