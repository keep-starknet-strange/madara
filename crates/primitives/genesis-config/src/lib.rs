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

impl From<FieldElement> for HexFelt {
    fn from(felt: FieldElement) -> Self {
        Self(felt)
    }
}

impl From<HexFelt> for FieldElement {
    fn from(hex_felt: HexFelt) -> Self {
        hex_felt.0
    }
}

impl From<Felt252Wrapper> for HexFelt {
    fn from(felt: Felt252Wrapper) -> HexFelt {
        HexFelt(felt.0)
    }
}

pub type ClassHash = HexFelt;
pub type ContractAddress = HexFelt;
pub type StorageKey = HexFelt;
pub type ContractStorageKey = (ContractAddress, StorageKey);
pub type StorageValue = HexFelt;

#[derive(Deserialize, Serialize)]
pub struct GenesisData {
    pub contract_classes: Vec<(ClassHash, ContractClass)>,
    pub sierra_class_hash_to_casm_class_hash: Vec<(ClassHash, ClassHash)>,
    pub contracts: Vec<(ContractAddress, ClassHash)>,
    pub predeployed_accounts: Vec<PredeployedAccount>,
    pub storage: Vec<(ContractStorageKey, StorageValue)>,
    pub fee_token_address: ContractAddress,
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
    pub fn base_path(&self) -> PathBuf {
        self.base_path.clone()
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ContractClass {
    Path { path: String, version: u8 },
    Class(StarknetContractClass),
}

/// A struct containing predeployed accounts info.
#[derive(Serialize, Deserialize)]
pub struct PredeployedAccount {
    pub contract_address: ContractAddress,
    pub class_hash: ClassHash,
    pub name: String,
    pub private_key: Option<Vec<u8>>,
}
