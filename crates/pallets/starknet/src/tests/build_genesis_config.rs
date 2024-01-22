use sp_runtime::BuildStorage;
use starknet_api::api_core::{ClassHash, ContractAddress};

use super::mock::default_mock;
use crate::GenesisConfig;

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
