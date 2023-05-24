use core::str::FromStr;
use std::path::PathBuf;
use std::{env, fs};

use blockifier::execution::contract_class::ContractClass;
use frame_support::bounded_vec;
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::MaxArraySize;
use sp_runtime::BoundedVec;
use starknet_crypto::{sign, FieldElement};

use super::constants::{ACCOUNT_PRIVATE_KEY, K};

pub fn get_contract_class(resource_path: &str) -> ContractClass {
    let cargo_dir = String::from(env!("CARGO_MANIFEST_DIR"));
    let full_path = cargo_dir + "/../../../resources/" + resource_path;
    println!("full_path: {}", full_path);
    let full_path: PathBuf = [full_path].iter().collect();
    println!("Present working directory of exe {}", env::current_dir().unwrap().display());
    let raw_contract_class = fs::read_to_string(full_path).unwrap();
    serde_json::from_str(&raw_contract_class).unwrap()
}

pub fn get_contract_class_wrapper(resource_path: &str) -> ContractClassWrapper {
    let contract_class = get_contract_class(resource_path);
    ContractClassWrapper::try_from(contract_class).unwrap()
}

pub fn sign_message_hash(hash: Felt252Wrapper) -> BoundedVec<Felt252Wrapper, MaxArraySize> {
    let signature = sign(
        &FieldElement::from_str(ACCOUNT_PRIVATE_KEY).unwrap(),
        &FieldElement::from(hash),
        &FieldElement::from_str(K).unwrap(),
    )
    .unwrap();
    bounded_vec!(signature.r.into(), signature.s.into())
}
