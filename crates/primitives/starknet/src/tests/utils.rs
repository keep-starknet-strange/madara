use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CachedState;
use cairo_lang_casm_contract_class::CasmContractClass;
use starknet_api::api_core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;

use crate::block::Block;
use crate::state::DictStateReader;

// Addresses.
pub const TEST_CONTRACT_ADDRESS: &str = "0x100";
pub const TEST_CONTRACT_ADDRESS_2: &str = "0x200";
pub const SECURITY_TEST_CONTRACT_ADDRESS: &str = "0x300";
pub const TEST_ACCOUNT_CONTRACT_ADDRESS: &str = "0x101";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_ADDRESS: &str = "0x102";
pub const TEST_SEQUENCER_ADDRESS: &str = "0x05a2b92d9a36509a3d651e7df99144a4ad8301e2caf42465ee6ab0451ae91882";
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
        (ClassHash(StarkFelt::try_from(TEST_CLASS_HASH).unwrap()), get_contract_class(TEST_CONTRACT_PATH, 0)),
        (
            ClassHash(StarkFelt::try_from(SECURITY_TEST_CLASS_HASH).unwrap()),
            get_contract_class(SECURITY_TEST_CONTRACT_PATH, 0),
        ),
    ]);

    // Two instances of a test contract and one instance of another (different) test contract.
    let address_to_class_hash = HashMap::from([
        (
            ContractAddress(PatriciaKey(StarkFelt::try_from(TEST_CONTRACT_ADDRESS).unwrap())),
            ClassHash(StarkFelt::try_from(TEST_CLASS_HASH).unwrap()),
        ),
        (
            ContractAddress(PatriciaKey(StarkFelt::try_from(TEST_CONTRACT_ADDRESS_2).unwrap())),
            ClassHash(StarkFelt::try_from(TEST_CLASS_HASH).unwrap()),
        ),
        (
            ContractAddress(PatriciaKey(StarkFelt::try_from(SECURITY_TEST_CONTRACT_ADDRESS).unwrap())),
            ClassHash(StarkFelt::try_from(SECURITY_TEST_CLASS_HASH).unwrap()),
        ),
    ]);

    CachedState::new(
        DictStateReader { class_hash_to_class, address_to_class_hash, ..Default::default() },
        Default::default(),
    )
}

pub fn get_contract_class(contract_path: &str, version: u8) -> ContractClass {
    let path: PathBuf = [contract_path].iter().collect();
    let raw_contract_class = fs::read_to_string(path).unwrap();
    if version == 0 {
        return ContractClass::V0(serde_json::from_str(&raw_contract_class).unwrap());
    } else if version == 1 {
        let casm_contract_class: CasmContractClass = serde_json::from_str(&raw_contract_class).unwrap();
        return ContractClass::V1(casm_contract_class.try_into().unwrap());
    }
    unimplemented!("version {} is not supported to get contract class from JSON", version);
}
