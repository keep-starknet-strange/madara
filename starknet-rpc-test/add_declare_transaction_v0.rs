extern crate starknet_rpc_test;
use std::sync::Arc;

use assert_matches::assert_matches;
use mp_transactions::BroadcastedDeclareTransactionV0;
use rstest::rstest;
use serde_json::json;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{BlockId, BlockTag, ContractClass};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::ARGENT_CONTRACT_ADDRESS;
use starknet_test_utils::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_test_utils::utils::get_transaction_receipt;

#[rstest]
#[tokio::test]
#[ignore]
async fn add_declare_transaction_v0_works(madara: &ThreadSafeMadaraClient) {
    let rpc = madara.get_starknet_client().await;

    let legacy_contract_class: LegacyContractClass = serde_json::from_reader(
        std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + "../starknet-rpc-test/contracts/proxy.json")
            .unwrap(),
    )
    .unwrap();
    let class_hash = legacy_contract_class.class_hash().unwrap();

    let tx = BroadcastedDeclareTransactionV0 {
        sender_address: FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(),
        // Amount used by the tx
        max_fee: FieldElement::from(482250u128),
        // No signature needed for DeclareV0
        signature: Vec::new(),
        contract_class: Arc::new(legacy_contract_class.compress().unwrap()),
        is_query: false,
    };

    let json_body = json!({
        "method": "madara_addDeclareTransactionV0",
        "params": [tx],
    });

    let block_number = {
        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_empty_block().await.unwrap();
        // Wasn't declared before
        assert!(rpc.get_class(BlockId::Tag(BlockTag::Latest), class_hash).await.is_err());
        madara_write_lock.call_rpc(json_body).await.unwrap();
        madara_write_lock.create_block_with_pending_txs().await.unwrap();
        rpc.block_number().await.unwrap()
    };

    // Tx was included in block
    let block = match rpc.get_block_with_tx_hashes(BlockId::Number(block_number)).await.unwrap() {
        starknet_core::types::MaybePendingBlockWithTxHashes::Block(b) => b,
        _ => panic!("This block is not pending"),
    };
    assert_eq!(block.transactions.len(), 1);
    let declare_tx_hash = block.transactions[0];

    // Wait for receipt to be available
    get_transaction_receipt(&rpc, declare_tx_hash).await.unwrap();
    // Is declared now
    let contract_class = rpc.get_class(BlockId::Number(block_number), class_hash).await.unwrap();
    assert_matches!(contract_class, ContractClass::Legacy(_));
}
