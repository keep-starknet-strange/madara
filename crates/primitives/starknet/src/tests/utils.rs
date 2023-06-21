use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CachedState;
use starknet_api::api_core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::{patricia_key, stark_felt};

use crate::block::Block;
use crate::state::DictStateReader;

// Addresses.
pub const TEST_CONTRACT_ADDRESS: &str = "0x100";
pub const TEST_CONTRACT_ADDRESS_2: &str = "0x200";
pub const SECURITY_TEST_CONTRACT_ADDRESS: &str = "0x300";
pub const TEST_ACCOUNT_CONTRACT_ADDRESS: &str = "0x101";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_ADDRESS: &str = "0x102";
pub const TEST_SEQUENCER_ADDRESS: &str = "0x1000";
pub const TEST_ERC20_CONTRACT_ADDRESS: &str = "0x1001";

// Class hashes.
pub const TEST_CLASS_HASH: &str = "0x110";
pub const TEST_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x111";
pub const TEST_EMPTY_CONTRACT_CLASS_HASH: &str = "0x112";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x113";
pub const SECURITY_TEST_CLASS_HASH: &str = "0x114";

// Paths.
pub const TEST_CONTRACT_PATH: &str = "../../../cairo-contracts/build/test.json";
pub const SECURITY_TEST_CONTRACT_PATH: &str = "../../../cairo-contracts/build/security_test.json";

pub const PEDERSEN_ZERO_HASH: &str = "0x49EE3EBA8C1600700EE1B87EB599F16716B0B1022947733551FDE4050CA6804";

impl Block {
    /// Creates a mock block.
    pub fn create_for_testing() -> Block {
        Block::default()
    }
}

pub fn create_test_state() -> CachedState<DictStateReader> {
    let class_hash_to_class = HashMap::from([
        (ClassHash(stark_felt!(TEST_CLASS_HASH)), get_contract_class(TEST_CONTRACT_PATH)),
        (ClassHash(stark_felt!(SECURITY_TEST_CLASS_HASH)), get_contract_class(SECURITY_TEST_CONTRACT_PATH)),
    ]);

    // Two instances of a test contract and one instance of another (different) test contract.
    let address_to_class_hash = HashMap::from([
        (ContractAddress(patricia_key!(TEST_CONTRACT_ADDRESS)), ClassHash(stark_felt!(TEST_CLASS_HASH))),
        (ContractAddress(patricia_key!(TEST_CONTRACT_ADDRESS_2)), ClassHash(stark_felt!(TEST_CLASS_HASH))),
        (
            ContractAddress(patricia_key!(SECURITY_TEST_CONTRACT_ADDRESS)),
            ClassHash(stark_felt!(SECURITY_TEST_CLASS_HASH)),
        ),
    ]);

    CachedState::new(DictStateReader { class_hash_to_class, address_to_class_hash, ..Default::default() })
}

pub fn get_contract_class(contract_path: &str) -> ContractClass {
    let path: PathBuf = [contract_path].iter().collect();
    let raw_contract_class = fs::read_to_string(path).unwrap();
    ContractClass::V0(serde_json::from_str(&raw_contract_class).unwrap()) // TODO (Greg) handle V1
}
