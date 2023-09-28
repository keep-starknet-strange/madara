use rstest::fixture;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeclareTransactionV1, BroadcastedTransaction,
    CompressedLegacyContractClass,
};
use starknet_ff::FieldElement;

use crate::{ExecutionStrategy, MadaraClient};
#[fixture]
pub async fn madara() -> MadaraClient {
    MadaraClient::new(ExecutionStrategy::Native).await
}

#[fixture]
pub fn compressed_contract_class() -> CompressedLegacyContractClass {
    let contract_class_bytes = include_bytes!("../../cairo-contracts/build/test.json");

    let contract_class: LegacyContractClass = serde_json::from_slice(contract_class_bytes).unwrap();
    let compressed_contract_class: CompressedLegacyContractClass = contract_class.compress().unwrap();

    compressed_contract_class
}

use crate::constants::ACCOUNT_CONTRACT;
#[fixture]
pub fn broadcasted_declare_txn_v1(compressed_contract_class: CompressedLegacyContractClass) -> BroadcastedTransaction {
    let txn = BroadcastedTransaction::Declare(BroadcastedDeclareTransaction::V1(BroadcastedDeclareTransactionV1 {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        is_query: true,
        contract_class: compressed_contract_class.into(),
    }));
    txn
}
