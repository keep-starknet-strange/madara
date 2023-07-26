use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;

use blockifier::execution::contract_class::ContractClass as StarknetContractClass;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::starknet_serde::get_contract_class;
use serde::{Deserialize, Serialize};

use crate::types::ContractStorageKeyWrapper;
use crate::GenesisConfig;

#[derive(Deserialize, Serialize, Debug)]
pub struct GenesisLoader {
    pub contract_classes: Vec<(ClassHashWrapper, ContractClass)>,
    pub contracts: Vec<(ContractAddressWrapper, ClassHashWrapper)>,
    pub storage: Vec<(ContractStorageKeyWrapper, Felt252Wrapper)>,
    pub fee_token_address: ContractAddressWrapper,
    pub chain_id: Felt252Wrapper,
    pub seq_addr_updated: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum ContractClass {
    Path { path: String, version: u8 },
    Class(StarknetContractClass),
}

impl<T: crate::Config> From<GenesisLoader> for GenesisConfig<T> {
    fn from(loader: GenesisLoader) -> Self {
        let classes = loader
            .contract_classes
            .into_iter()
            .map(|(hash, class)| match class {
                ContractClass::Path { path, version } => {
                    (hash, get_contract_class(&read_file_to_string(&path), version))
                }
                ContractClass::Class(class) => (hash, class),
            })
            .collect::<Vec<_>>();

        GenesisConfig {
            contracts: loader.contracts,
            contract_classes: classes,
            storage: loader.storage,
            fee_token_address: loader.fee_token_address,
            chain_id: loader.chain_id,
            seq_addr_updated: loader.seq_addr_updated,
            ..Default::default()
        }
    }
}

pub fn read_file_to_string(path: &str) -> String {
    let cargo_dir = String::from(env!("CARGO_MANIFEST_DIR"));
    let path: PathBuf = [cargo_dir + "/" + path].iter().collect();
    fs::read_to_string(path).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_loader() {
        // When
        let loader: GenesisLoader = serde_json::from_str(&read_file_to_string("src/tests/mock/genesis.json")).unwrap();

        // Then
        assert_eq!(13, loader.contract_classes.len());
    }
}
