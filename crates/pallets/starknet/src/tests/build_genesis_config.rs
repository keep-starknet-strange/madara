use mp_genesis_config::{GenesisData, GenesisLoader};
use sp_runtime::traits::{Block as BlockT, Hash as HashT, Header as HeaderT, Zero};
use sp_runtime::{BuildStorage, Storage};
use starknet_api::api_core::{ClassHash, ContractAddress};

use super::mock::default_mock;
use super::utils::get_contract_class;
use crate::{Config, ContractClasses, GenesisConfig, Pallet};

#[test]
fn works_when_sierra_clash_hash_in_mapping_is_known() {
    // create empty storage compatible with genesis file
    let mut t = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();

    // create genesis config
    let genesis: GenesisConfig<default_mock::MockRuntime> = GenesisConfig {
        sierra_to_casm_class_hash: vec![(ClassHash(1u8.into()), ClassHash(42u8.into()))],
        contract_classes: vec![(ClassHash(1u8.into()), get_contract_class("ERC20.json", 0))],
        ..Default::default()
    };

    // populate storage
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

#[test]
fn check_genesis_storage() {
    // setup
    let project_root = project_root::get_project_root().unwrap().join("configs/");
    let genesis_path = project_root.join("genesis-assets/").join("genesis.json");
    let genesis_file_content = std::fs::read_to_string(&genesis_path).unwrap();

    let genesis_data: GenesisData = serde_json::from_str(&genesis_file_content).unwrap();
    let genesis_loader = GenesisLoader::new(project_root.clone(), genesis_data.clone());
    let genesis_loader_2 = GenesisLoader::new(project_root.clone(), genesis_data.clone());

    assert_eq!(genesis_loader, genesis_loader_2);

    // test
    let mut t: Storage = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();
    let mut t_2: Storage = frame_system::GenesisConfig::<default_mock::MockRuntime>::default().build_storage().unwrap();

    let genesis: GenesisConfig<default_mock::MockRuntime> = genesis_loader.into();
    let genesis_2: GenesisConfig<default_mock::MockRuntime> = genesis_loader_2.into();

    genesis.assimilate_storage(&mut t).unwrap();

    genesis_2.assimilate_storage(&mut t_2).unwrap();

    assert_eq!(t.top, t_2.top);
    assert_eq!(t.children_default, t_2.children_default);
}
