use std::fmt;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;

use blockifier::execution::contract_class::ContractClass as StarknetContractClass;
use derive_more::Constructor;
use mp_felt::Felt252Wrapper;
use serde::{Deserialize, Serialize, Serializer};
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
    fn from(element: FieldElement) -> Self {
        Self(element)
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
    pub contracts: Vec<(ContractAddress, ClassHash)>,
    pub predeployed_accounts: Vec<PredeployedAccount>,
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
    pub address: FieldElement,
    pub class_hash: FieldElement,
    pub name: String,
	#[serde(serialize_with = "buffer_to_hex")]
	pub private_key: Option<Vec<u8>>,
	pub public_key: HexFelt,
}

pub fn buffer_to_hex<S>(buffer: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer
{
	if let Some(inner_buffer) = buffer {
		let hex_string = format!("0x{}",hex::encode(inner_buffer));
		serializer.serialize_str(&*hex_string)
	} else {
		serializer.serialize_none()
	}
}
