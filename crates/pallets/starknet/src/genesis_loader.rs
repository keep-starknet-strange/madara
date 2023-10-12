use std::fmt;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;

use blockifier::execution::contract_class::ContractClass as StarknetContractClass;
use derive_more::Constructor;
use mp_felt::Felt252Wrapper;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet_core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use crate::GenesisConfig;

/// A wrapper for FieldElement that implements serde's Serialize and Deserialize for hex strings.
#[serde_as]
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct HexFelt(#[serde_as(as = "UfeHex")] pub FieldElement);

impl fmt::LowerHex for HexFelt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;

        fmt::LowerHex::fmt(&val, f)
    }
}

impl From<Felt252Wrapper> for HexFelt {
    fn from(felt: Felt252Wrapper) -> HexFelt {
        HexFelt(felt.0)
    }
}

type ClassHash = HexFelt;
type ContractAddress = HexFelt;
type StorageKey = HexFelt;
type ContractStorageKey = (ContractAddress, StorageKey);
type StorageValue = HexFelt;

#[derive(Deserialize, Serialize)]
pub struct GenesisData {
    pub contract_classes: Vec<(ClassHash, ContractClass)>,
    pub contracts: Vec<(ContractAddress, ClassHash)>,
    pub storage: Vec<(ContractStorageKey, StorageValue)>,
    pub fee_token_address: ContractAddress,
    pub seq_addr_updated: bool,
}

#[derive(Constructor)]
pub struct GenesisLoader {
    base_path: PathBuf,
    data: GenesisData,
}

impl GenesisLoader {
    pub fn data(&self) -> &GenesisData {
        &self.data
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ContractClass {
    Path { path: String, version: u8 },
    Class(StarknetContractClass),
}

impl<T: crate::Config> From<GenesisLoader> for GenesisConfig<T> {
    fn from(loader: GenesisLoader) -> Self {
        let contract_classes = loader
            .data
            .contract_classes
            .into_iter()
            .map(|(hash, class)| {
                let hash = Felt252Wrapper(hash.0).into();
                match class {
                    ContractClass::Path { path, version } => (
                        hash,
                        read_contract_class_from_json(
                            &std::fs::read_to_string(loader.base_path.join(path)).expect(
                                "Some contract is missing in the config folder. Try to run `madara setup` before \
                                 opening an issue.",
                            ),
                            version,
                        ),
                    ),
                    ContractClass::Class(class) => (hash, class),
                }
            })
            .collect::<Vec<_>>();
        let contracts = loader
            .data
            .contracts
            .into_iter()
            .map(|(address, hash)| {
                let address = Felt252Wrapper(address.0).into();
                let hash = Felt252Wrapper(hash.0).into();
                (address, hash)
            })
            .collect::<Vec<_>>();
        let storage = loader
            .data
            .storage
            .into_iter()
            .map(|(key, value)| {
                let key = (Felt252Wrapper(key.0.0).into(), Felt252Wrapper(key.1.0).into());
                let value = Felt252Wrapper(value.0).into();
                (key, value)
            })
            .collect::<Vec<_>>();
        let fee_token_address = Felt252Wrapper(loader.data.fee_token_address.0).into();

        GenesisConfig {
            contracts,
            contract_classes,
            storage,
            fee_token_address,
            seq_addr_updated: loader.data.seq_addr_updated,
            ..Default::default()
        }
    }
}

/// Create a `ContractClass` from a JSON string
///
/// This function takes a JSON string (`json_str`) containing the JSON representation of a
/// ContractClass
///
/// `ContractClassV0` can be read directly from the JSON because the Serde methods have been
/// implemented in the blockifier
///
/// `ContractClassV1` needs to be read in Casm and then converted to Contract Class V1
pub(crate) fn read_contract_class_from_json(json_str: &str, version: u8) -> StarknetContractClass {
    if version == 0 {
        return StarknetContractClass::V0(
            serde_json::from_str(json_str).expect("`json_str` should be deserializable into the correct ContracClass"),
        );
    } else if version == 1 {
        let casm_contract_class: cairo_lang_casm_contract_class::CasmContractClass =
            serde_json::from_str(json_str).expect("`json_str` should be deserializable into the CasmContracClass");
        return StarknetContractClass::V1(
            casm_contract_class.try_into().expect("the CasmContractClass should produce a valid ContractClassV1"),
        );
    }
    unimplemented!("version {} is not supported to get contract class from JSON", version);
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
        let loader: GenesisData = serde_json::from_str(include_str!("./tests/mock/genesis.json")).unwrap();

        // Then
        assert_eq!(13, loader.contract_classes.len());
    }

    #[test]
    fn test_serialize_loader() {
        // Given
        let class: ContractClass = ContractClass::Path { path: "cairo-contracts/ERC20.json".into(), version: 0 };

        let class_hash = FieldElement::from(1u8).into();
        let contract_address = FieldElement::from(2u8).into();
        let storage_key = FieldElement::from(3u8).into();
        let storage_value = FieldElement::from(4u8).into();
        let fee_token_address = FieldElement::from(5u8).into();

        let genesis_loader = GenesisData {
            contract_classes: vec![(class_hash, class)],
            contracts: vec![(contract_address, class_hash)],
            storage: vec![((contract_address, storage_key), storage_value)],
            fee_token_address,
            seq_addr_updated: false,
        };

        // When
        let serialized_loader = serde_json::to_string(&genesis_loader).unwrap();

        // Then
        let expected = r#"{"contract_classes":[["0x1",{"path":"cairo-contracts/ERC20.json","version":0}]],"contracts":[["0x2","0x1"]],"storage":[[["0x2","0x3"],"0x4"]],"fee_token_address":"0x5","seq_addr_updated":false}"#;
        assert_eq!(expected, serialized_loader);
    }
}
