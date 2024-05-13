use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{
    BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedInvokeTransactionV1, BroadcastedTransaction,
    StarknetError,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::constants::{ACCOUNT_CONTRACT_ADDRESS, MULTIPLY_TEST_CONTRACT_ADDRESS, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::is_good_error_code;

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

    let tx = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap(),
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
            nonce: FieldElement::ONE,
            ..tx
        }));

    let estimates = rpc
        .estimate_fee(&vec![invoke_transaction, invoke_transaction_2], vec![], BlockId::Tag(BlockTag::Latest))
        .await?;

    // TODO: instead execute the tx and check that the actual fee are the same as the estimated ones
    assert_eq!(estimates.len(), 2);
    // TODO: use correct values when we implement estimate fee correctly
    assert_eq!(estimates[0].overall_fee, FieldElement::from(48080u128));
    assert_eq!(estimates[1].overall_fee, FieldElement::from(48080u128));
    // https://starkscan.co/block/5
    assert_eq!(estimates[0].gas_consumed, FieldElement::ZERO);
    assert_eq!(estimates[1].gas_consumed, FieldElement::ZERO);

    Ok(())
}
