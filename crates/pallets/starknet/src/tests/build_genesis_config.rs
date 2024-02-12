use sp_runtime::BuildStorage;
use starknet_api::api_core::{ClassHash, ContractAddress};

use super::mock::default_mock;
use super::utils::get_contract_class;
use crate::GenesisConfig;

#[test]
fn works_when_sierra_clash_hash_in_mapping_is_known() {
    let mut t = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();
    let genesis: GenesisConfig<default_mock::MockRuntime> = GenesisConfig {
        sierra_to_casm_class_hash: vec![(ClassHash(1u8.into()), ClassHash(42u8.into()))],
        contract_classes: vec![(ClassHash(1u8.into()), get_contract_class("ERC20.json", 0))],
        ..Default::default()
    };
    genesis.assimilate_storage(&mut t).unwrap();
}

#[test]
#[should_panic(expected = "does not exist in contract_classes")]
fn fails_when_only_casm_clash_hash_in_mapping_is_known() {
    let mut t = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();
    let genesis: GenesisConfig<default_mock::MockRuntime> = GenesisConfig {
        sierra_to_casm_class_hash: vec![(ClassHash(1u8.into()), ClassHash(42u8.into()))],
        contract_classes: vec![(ClassHash(42u8.into()), get_contract_class("ERC20.json", 0))],
        ..Default::default()
    };
    genesis.assimilate_storage(&mut t).unwrap();
}

#[test]
#[should_panic(expected = "does not exist in contract_classes")]
fn fail_with_unknown_class_hash_in_sierra_mappings() {
    let mut t = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();
    let genesis: GenesisConfig<default_mock::MockRuntime> = GenesisConfig {
        sierra_to_casm_class_hash: vec![(ClassHash(1u8.into()), ClassHash(42u8.into()))],
        ..Default::default()
    };
    genesis.assimilate_storage(&mut t).unwrap();
}

#[test]
#[should_panic(expected = "does not exist in contract_classes")]
fn fail_with_unknown_class_hash_in_contracts() {
    let mut t = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();
    let genesis: GenesisConfig<default_mock::MockRuntime> =
        GenesisConfig { contracts: vec![(ContractAddress(1u8.into()), ClassHash(42u8.into()))], ..Default::default() };
    genesis.assimilate_storage(&mut t).unwrap();
}
