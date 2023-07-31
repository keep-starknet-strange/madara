use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;

use blockifier::execution::contract_class::ContractClass as StarknetContractClass;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::starknet_serde::get_contract_class;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet_core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use crate::types::ContractStorageKeyWrapper;
use crate::GenesisConfig;

/// A wrapper for FieldElement that implements serde's Serialize and Deserialize for hex strings.
#[serde_as]
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct HexFelt(#[serde_as(as = "UfeHex")] pub FieldElement);

type ClassHash = HexFelt;
type ContractAddress = HexFelt;
type StorageKey = HexFelt;
type ContractStorageKey = (ContractAddress, StorageKey);
type StorageValue = HexFelt;

#[derive(Deserialize, Serialize)]
pub struct GenesisLoader {
    pub contract_classes: Vec<(ClassHash, ContractClass)>,
    pub contracts: Vec<(ContractAddress, ClassHash)>,
    pub storage: Vec<(ContractStorageKey, StorageValue)>,
    pub fee_token_address: ContractAddress,
    pub seq_addr_updated: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ContractClass {
    Path { path: String, version: u8 },
    Class(StarknetContractClass),
}

impl<T: crate::Config> From<GenesisLoader> for GenesisConfig<T> {
    fn from(loader: GenesisLoader) -> Self {
        let contract_classes = loader
            .contract_classes
            .into_iter()
            .map(|(hash, class)| {
                let hash = unsafe { std::mem::transmute::<ClassHash, ClassHashWrapper>(hash) };
                match class {
                    ContractClass::Path { path, version } => {
                        (hash, get_contract_class(&read_file_to_string(&path), version))
                    }
                    ContractClass::Class(class) => (hash, class),
                }
            })
            .collect::<Vec<_>>();
        let contracts = loader
            .contracts
            .into_iter()
            .map(|(address, hash)| {
                let address = unsafe { std::mem::transmute::<ContractAddress, ContractAddressWrapper>(address) };
                let hash = unsafe { std::mem::transmute::<ClassHash, ClassHashWrapper>(hash) };
                (address, hash)
            })
            .collect::<Vec<_>>();
        let storage = loader
            .storage
            .into_iter()
            .map(|(key, value)| {
                let key = unsafe { std::mem::transmute::<ContractStorageKey, ContractStorageKeyWrapper>(key) };
                let value = unsafe { std::mem::transmute::<StorageValue, Felt252Wrapper>(value) };
                (key, value)
            })
            .collect::<Vec<_>>();
        let fee_token_address =
            unsafe { std::mem::transmute::<ContractAddress, ContractAddressWrapper>(loader.fee_token_address) };

        GenesisConfig {
            contracts,
            contract_classes,
            storage,
            fee_token_address,
            seq_addr_updated: loader.seq_addr_updated,
            ..Default::default()
        }
    }
}

pub fn read_file_to_string(path: &str) -> String {
    let workspace = std::process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("Failed to execute cargo locate-project command")
        .stdout;
    let mut dir = PathBuf::from(std::str::from_utf8(&workspace).unwrap().trim());
    dir.pop();
    dir.push(path);
    fs::read_to_string(dir).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    impl From<FieldElement> for HexFelt {
        fn from(element: FieldElement) -> Self {
            Self(element)
        }
    }

    #[test]
    fn test_deserialize_loader() {
        // When
        let loader: GenesisLoader =
            serde_json::from_str(&read_file_to_string("crates/pallets/starknet/src/tests/mock/genesis.json")).unwrap();

        // Then
        assert_eq!(13, loader.contract_classes.len());
    }

    #[test]
    fn test_serialize_loader() {
        // Given
        let class: ContractClass =
            ContractClass::Path { path: "./cairo-contracts/build/ERC20.json".into(), version: 0 };

        let class_hash = FieldElement::from(1u8).into();
        let contract_address = FieldElement::from(2u8).into();
        let storage_key = FieldElement::from(3u8).into();
        let storage_value = FieldElement::from(4u8).into();
        let fee_token_address = FieldElement::from(5u8).into();

        let genesis_loader = GenesisLoader {
            contract_classes: vec![(class_hash, class)],
            contracts: vec![(contract_address, class_hash)],
            storage: vec![((contract_address, storage_key), storage_value)],
            fee_token_address,
            seq_addr_updated: false,
        };

        // When
        let serialized_loader = serde_json::to_string(&genesis_loader).unwrap();

        // Then
        let expected = r#"{"contract_classes":[["0x1",{"path":"./cairo-contracts/build/ERC20.json","version":0}]],"contracts":[["0x2","0x1"]],"storage":[[["0x2","0x3"],"0x4"]],"fee_token_address":"0x5","seq_addr_updated":false}"#;
        assert_eq!(expected, serialized_loader);
    }
}
