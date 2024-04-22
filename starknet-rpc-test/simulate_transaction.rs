use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Call, ConnectedAccount};
use starknet_core::types::{
    BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedInvokeTransactionV1, BroadcastedTransaction,
    SimulationFlag, StarknetError,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{Provider, ProviderError};
use starknet_rpc_test::constants::{
    ACCOUNT_CONTRACT_ADDRESS, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, TEST_CONTRACT_ADDRESS,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, is_good_error_code, AccountActions};

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
            is_query: false,
        }));

    let simulate_transaction_error =
        rpc.simulate_transactions(BlockId::Hash(FieldElement::ZERO), &[ok_invoke_transaction], []).await.unwrap_err();
    assert_matches!(simulate_transaction_error, ProviderError::StarknetError(StarknetError::BlockNotFound));

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_max_fee_too_big(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let ok_invoke_transaction =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            max_fee: FieldElement::from_hex_be("0x100000000000000000000000000000000").unwrap(), // u128::MAX + 1
            signature: vec![],
            nonce: FieldElement::ZERO,
            sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap(),
            calldata: vec![
                FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
                get_selector_from_name("sqrt").unwrap(),
                FieldElement::from_hex_be("1").unwrap(),
                FieldElement::from(81u8),
            ],
            is_query: false,
        }));

    let simulate_transaction_error =
        rpc.simulate_transactions(BlockId::Tag(BlockTag::Latest), &[ok_invoke_transaction], []).await.unwrap_err();
    assert!(is_good_error_code(&simulate_transaction_error, 500));

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
            is_query: false,
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
            is_query: false,
        }));

    let simulate_transaction_error = rpc
        .simulate_transactions(BlockId::Tag(BlockTag::Latest), &[bad_invoke_transaction, ok_invoke_transaction], [])
        .await
        .unwrap_err();
    assert!(is_good_error_code(&simulate_transaction_error, 40));

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_on_no_validate(madara: &ThreadSafeMadaraClient) {
    let rpc = madara.get_starknet_client().await;

    let sender_address = FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap();

    let mut madara_write_lock = madara.write().await;
    let _ = madara_write_lock.create_empty_block().await;

    let tx = BroadcastedInvokeTransactionV1 {
        sender_address,
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        max_fee: FieldElement::from(100_000u128),
        signature: vec![],
        nonce: FieldElement::ZERO,
        is_query: false,
    };

    let invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(tx.clone()));

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            nonce: FieldElement::ONE,
            ..tx
        }));

    let simulations = rpc
        .simulate_transactions(BlockId::Tag(BlockTag::Latest), &[invoke_transaction, invoke_transaction_2], [])
        .await
        .unwrap();

    assert_eq!(simulations.len(), 2);
    // TODO: check again when implemented correctly
    // assert_eq!(simulations[0].fee_estimation.gas_consumed, FieldElement::ZERO);
    // assert_eq!(simulations[0].fee_estimation.overall_fee, FieldElement::from(210u128));
    // assert_eq!(simulations[0].fee_estimation.gas_price, FieldElement::ZERO);
}

#[rstest]
#[tokio::test]
async fn works_ok_on_validate_with_signature(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let nonce = funding_account.get_nonce().await?;
    let max_fee = FieldElement::from(100_000u128);

    let calls = vec![Call {
        to: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        selector: get_selector_from_name("sqrt").unwrap(),
        calldata: vec![FieldElement::from(81u8)],
    }];
    let tx = funding_account.prepare_invoke(calls, nonce, max_fee, false).await;

    let invoke_transaction = BroadcastedTransaction::Invoke(tx);

    let simulations = rpc.simulate_transactions(BlockId::Tag(BlockTag::Latest), &[invoke_transaction], []).await?;

    assert_eq!(simulations.len(), 1);
    // TODO: check again when implemented correctly
    // assert_eq!(simulations[0].fee_estimation.gas_consumed, FieldElement::ZERO);
    // assert_eq!(simulations[0].fee_estimation.overall_fee, FieldElement::from(240u128));
    // assert_eq!(simulations[0].fee_estimation.gas_price, FieldElement::ZERO);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_on_validate_without_signature_with_skip_validate(
    madara: &ThreadSafeMadaraClient,
) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let nonce = funding_account.get_nonce().await?;
    let max_fee = FieldElement::from(100_000u128);

    let calls = vec![Call {
        to: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        selector: get_selector_from_name("sqrt").unwrap(),
        calldata: vec![FieldElement::from(81u8)],
    }];
    let tx = funding_account.prepare_invoke(calls, nonce, max_fee, false).await;

    let invoke_transaction = BroadcastedTransaction::Invoke(tx);

    let simulations = rpc
        .simulate_transactions(BlockId::Tag(BlockTag::Latest), &[invoke_transaction], [SimulationFlag::SkipValidate])
        .await?;

    assert_eq!(simulations.len(), 1);
    // TODO: check again when implemented correctly
    // assert_eq!(simulations[0].fee_estimation.gas_consumed, FieldElement::ZERO);
    // assert_eq!(simulations[0].fee_estimation.overall_fee, FieldElement::from(220u128));
    // assert_eq!(simulations[0].fee_estimation.gas_price, FieldElement::ZERO);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_without_max_fee_with_skip_fee_charge(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let sender_address = FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap();

    let tx = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::from(0u8),
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address,
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        is_query: false,
    };

    let invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(tx.clone()));

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction::V1(BroadcastedInvokeTransactionV1 {
            nonce: FieldElement::ONE,
            ..tx
        }));

    let simulations = rpc
        .simulate_transactions(
            BlockId::Tag(BlockTag::Latest),
            &[invoke_transaction, invoke_transaction_2],
            [SimulationFlag::SkipFeeCharge],
        )
        .await?;

    assert_eq!(simulations.len(), 2);
    // TODO: check again when implemented correctly
    // assert_eq!(simulations[0].fee_estimation.gas_consumed, FieldElement::ZERO);
    // assert_eq!(simulations[0].fee_estimation.overall_fee, FieldElement::from(210u128));
    // assert_eq!(simulations[0].fee_estimation.gas_price, FieldElement::ZERO);

    Ok(())
}
