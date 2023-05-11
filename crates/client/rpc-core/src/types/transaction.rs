use serde::{Deserialize, Serialize};

use super::{
    ContractAddress, ContractClassHash, FieldElement, MaxFee, Nonce, Signature, StarknetTransactionHash, Version,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct CommonTxProperties {
    pub max_fee: MaxFee,
    pub version: Version,
    pub signature: Signature,
    pub nonce: Nonce,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "DECLARE")]
    Declare(BroadcastedDeclareTransactionV2),
}

#[derive(Serialize, Clone, Debug, Deserialize, PartialEq)]
pub struct BroadcastedDeclareTransactionV2 {
    #[serde(flatten)]
    pub common: CommonTxProperties,
    pub compiled_class_hash: FieldElement,
    pub contract_class: super::DeprecatedContractClass,
    pub sender_address: ContractAddress,
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct AddDeclareTransactionOutput {
    pub transaction_hash: StarknetTransactionHash,
    pub class_hash: ContractClassHash,
}
