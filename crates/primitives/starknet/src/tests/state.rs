use blockifier::execution::contract_class::{ContractClass, ContractClassV0};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::StateReader;
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;

use crate::state::*;

#[test]
fn test_get_storage_at() {
    let mut state = DictStateReader::default();

    let address = ContractAddress::default();
    let key = StorageKey::default();
    let value = StarkFelt::default();
    let storage_key = (address, key);

    state.storage_view.insert(storage_key, value);

    let result = state.get_storage_at(address, key).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_get_nonce_at() {
    let mut state = DictStateReader::default();

    let address = ContractAddress::default();
    let nonce = Nonce::default();

    state.address_to_nonce.insert(address, nonce);

    let result = state.get_nonce_at(address).unwrap();
    assert_eq!(result, nonce);
}

#[test]
fn test_get_contract_class() {
    let mut state = DictStateReader::default();

    let class_hash = ClassHash::default();
    let contract_class = ContractClass::V0(ContractClassV0::default()); // Replace with an actual ContractClass instance

    state.class_hash_to_class.insert(class_hash, contract_class.clone());

    let result = state.get_compiled_contract_class(&class_hash).unwrap();
    assert_eq!(result, contract_class);
}

#[test]
fn test_get_class_hash_at() {
    let mut state = DictStateReader::default();

    let address = ContractAddress::default();
    let class_hash = ClassHash::default();

    state.address_to_class_hash.insert(address, class_hash);

    let result = state.get_class_hash_at(address).unwrap();
    assert_eq!(result, class_hash);
}

#[test]
fn test_get_contract_class_undeclared_class_hash() {
    let mut state = DictStateReader::default();

    let undeclared_class_hash = ClassHash::default();

    let result = state.get_compiled_contract_class(&undeclared_class_hash);
    assert!(result.is_err());

    if let Err(StateError::UndeclaredClassHash(hash)) = result {
        assert_eq!(hash, undeclared_class_hash);
    } else {
        panic!("Unexpected error");
    }
}
