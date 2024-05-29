extern crate starknet_rpc_test;
use anyhow::Error;
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{
    BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedInvokeTransactionV1, BroadcastedTransaction,
    FeeEstimate, MaybePendingTransactionReceipt, StarknetError, TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::constants::{ACCOUNT_CONTRACT_ADDRESS,MULTIPLY_TEST_CONTRACT_ADDRESS, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{get_transaction_receipt, is_good_error_code};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let ok_invoke_transaction =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
            sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap(),
            calldata: vec![
                FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
                get_selector_from_name("sqrt").unwrap(),
                FieldElement::from_hex_be("1").unwrap(),
                FieldElement::from(81u8),
            ],
            is_query: true,
        }));

    assert_matches!(
        rpc.estimate_fee(&vec![ok_invoke_transaction], vec![], BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetError::BlockNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_one_txn_cannot_be_executed(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let bad_invoke_transaction =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            max_fee: FieldElement::default(),
            nonce: FieldElement::ZERO,
            sender_address: FieldElement::default(),
            signature: vec![],
            calldata: vec![FieldElement::from_hex_be("0x0").unwrap()],
            is_query: true,
        }));

    let ok_invoke_transaction =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
            sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap(),
            calldata: vec![
                FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
                get_selector_from_name("sqrt").unwrap(),
                FieldElement::from_hex_be("1").unwrap(),
                FieldElement::from(81u8),
            ],
            is_query: true,
        }));

    let estimate_fee_error = rpc
        .estimate_fee(&vec![bad_invoke_transaction, ok_invoke_transaction], vec![], BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap_err();
    assert!(is_good_error_code(&estimate_fee_error, 40));

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let sender_address = FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap();
    let nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), sender_address).await.unwrap();

    let mut tx = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::from_hex_be("0xfffffffffff").unwrap(),
        signature: vec![],
        nonce,
        sender_address,
        calldata: vec![
            FieldElement::from_hex_be(MULTIPLY_TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("multiply").unwrap(),
            FieldElement::TWO,
            FieldElement::from_hex_be("2").unwrap(),
            FieldElement::from_hex_be("5").unwrap(),
        ],
        is_query: true,
    };

    let invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(tx.clone()));

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            nonce: nonce + FieldElement::ONE,
            calldata: vec![
                FieldElement::from_hex_be(MULTIPLY_TEST_CONTRACT_ADDRESS).unwrap(),
                get_selector_from_name("multiply").unwrap(),
                FieldElement::TWO,
                FieldElement::from_hex_be("3").unwrap(),
                FieldElement::from_hex_be("5").unwrap(),
            ],
            ..tx.clone()
        }));

    let estimates = rpc
        .estimate_fee(&vec![invoke_transaction, invoke_transaction_2], vec![], BlockId::Tag(BlockTag::Latest))
        .await?;

    assert_eq!(estimates.len(), 2);

    tx.is_query = false;
    let invoke_transaction = BroadcastedInvokeTransaction::V1(tx.clone());
    let invoke_transaction_2 = BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
        nonce: FieldElement::ONE,
        calldata: vec![
            FieldElement::from_hex_be(MULTIPLY_TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("multiply").unwrap(),
            FieldElement::TWO,
            FieldElement::from_hex_be("3").unwrap(),
            FieldElement::from_hex_be("5").unwrap(),
        ],
        ..tx.clone()
    });
    let executed_tx_1 = rpc.add_invoke_transaction(invoke_transaction).await?;
    let executed_tx_2 = rpc.add_invoke_transaction(invoke_transaction_2).await?;

    madara.write().await.create_block_with_pending_txs().await?;

    let receipt_1 = get_transaction_receipt(&rpc, executed_tx_1.transaction_hash).await?;
    let receipt_2 = get_transaction_receipt(&rpc, executed_tx_2.transaction_hash).await?;

    let match_estimate_and_receipt = |estimate: FeeEstimate,
                                      receipt: MaybePendingTransactionReceipt|
     -> Result<(), anyhow::Error> {
        match receipt {
            MaybePendingTransactionReceipt::PendingReceipt(_) => Err(Error::msg("Transaction should not be pending")),
            MaybePendingTransactionReceipt::Receipt(receipt) => match receipt {
                TransactionReceipt::Invoke(receipt) => {
                    assert_eq!(estimate.overall_fee, receipt.actual_fee.amount);
                    Ok(())
                }
                _ => Err(Error::msg("Transaction should be an invoke transaction")),
            },
        }
    };

    match_estimate_and_receipt(estimates[0].clone(), receipt_1)?;
    match_estimate_and_receipt(estimates[1].clone(), receipt_2)?;

    Ok(())
}
