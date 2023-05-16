use serde::{Deserialize, Serialize};

use super::{
    deserialize_field_element, deserialize_vec_field_element, ContractAddress, ContractClassHash, FieldElement, MaxFee,
    Nonce, Signature, StarknetTransactionHash, Version,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct CommonTxProperties {
    pub max_fee: MaxFee,
    pub version: Version,
    #[serde(deserialize_with = "deserialize_vec_field_element")]
    pub signature: Signature,
    pub nonce: Nonce,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "DECLARE")]
    Declare(BroadcastedDeclareTransactionV2),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum DeployAccountTransactionType {
    #[default]
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount,
}

#[derive(Serialize, Clone, Debug, Deserialize, PartialEq, Default)]
pub struct BroadcastedDeployAccountTransaction {
    #[serde(flatten)]
    pub common: CommonTxProperties,
    #[serde(rename = "type")]
    pub _type: DeployAccountTransactionType,
    #[serde(deserialize_with = "deserialize_field_element")]
    pub contract_address_salt: FieldElement,
    pub constructor_calldata: Vec<FieldElement>,
    #[serde(deserialize_with = "deserialize_field_element")]
    pub class_hash: FieldElement,
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct AddDeployAccountTransactionOutput {
    pub transaction_hash: StarknetTransactionHash,
    pub contract_address: ContractAddress,
}
