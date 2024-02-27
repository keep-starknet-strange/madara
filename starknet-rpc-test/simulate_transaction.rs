extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Call, ConnectedAccount};
use starknet_core::types::{
    BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedTransaction, SimulationFlag, StarknetError,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ACCOUNT_CONTRACT, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let ok_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        is_query: false,
    });

    assert_matches!(
        rpc.simulate_transactions(BlockId::Hash(FieldElement::ZERO),&[ok_invoke_transaction], []).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_max_fee_too_big(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let ok_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::from_hex_be("0x100000000000000000000000000000000").unwrap(), // u128::MAX + 1
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        is_query: false,
    });

    assert_matches!(
        rpc.simulate_transactions(BlockId::Tag(BlockTag::Latest), &[ok_invoke_transaction], []).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Unknown(500), message })) if message == "Internal server error"
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_one_txn_cannot_be_executed(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let bad_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::default(),
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::default(),
        signature: vec![],
        calldata: vec![FieldElement::from_hex_be("0x0").unwrap()],
        is_query: false,
    });

    let ok_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        is_query: false,
    });

    assert_matches!(
        rpc.simulate_transactions(BlockId::Tag(BlockTag::Latest),&[
            bad_invoke_transaction,
            ok_invoke_transaction,
        ],[] ).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_on_no_validate(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let sender_address = FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap();

    let mut madara_write_lock = madara.write().await;
    let _ = madara_write_lock.create_empty_block().await;

    let tx = BroadcastedInvokeTransaction {
        sender_address,
        calldata: vec![
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("sqrt").unwrap(),
            FieldElement::from_hex_be("1").unwrap(),
            FieldElement::from(81u8),
        ],
        max_fee: FieldElement::from(210u16),
        signature: vec![],
        nonce: FieldElement::ZERO,
        is_query: false,
    };

    let invoke_transaction = BroadcastedTransaction::Invoke(tx.clone());

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction { nonce: FieldElement::ONE, ..tx });

    let simulations = rpc
        .simulate_transactions(BlockId::Tag(BlockTag::Latest), &[invoke_transaction, invoke_transaction_2], [])
        .await?;

    assert_eq!(simulations.len(), 2);
    assert_eq!(simulations[0].fee_estimation.gas_consumed, 0);
    assert_eq!(simulations[0].fee_estimation.overall_fee, 210);
    assert_eq!(simulations[0].fee_estimation.gas_price, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_on_validate_with_signature(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let nonce = funding_account.get_nonce().await?;
    let max_fee = FieldElement::from(1000u16);

    let calls = vec![Call {
        to: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        selector: get_selector_from_name("sqrt").unwrap(),
        calldata: vec![FieldElement::from(81u8)],
    }];
    let tx = funding_account.prepare_invoke(calls, nonce, max_fee, false).await;

    let invoke_transaction = BroadcastedTransaction::Invoke(tx);

    let simulations = rpc.simulate_transactions(BlockId::Tag(BlockTag::Latest), &[invoke_transaction], []).await?;

    assert_eq!(simulations.len(), 1);
    assert_eq!(simulations[0].fee_estimation.gas_consumed, 0);
    assert_eq!(simulations[0].fee_estimation.overall_fee, 240);
    assert_eq!(simulations[0].fee_estimation.gas_price, 0);

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
    let max_fee = FieldElement::from(1000u16);

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
    assert_eq!(simulations[0].fee_estimation.gas_consumed, 0);
    assert_eq!(simulations[0].fee_estimation.overall_fee, 220);
    assert_eq!(simulations[0].fee_estimation.gas_price, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok_without_max_fee_with_skip_fee_charge(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let sender_address = FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap();

    let tx = BroadcastedInvokeTransaction {
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

    let invoke_transaction = BroadcastedTransaction::Invoke(tx.clone());

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction { nonce: FieldElement::ONE, ..tx });

    let simulations = rpc
        .simulate_transactions(
            BlockId::Tag(BlockTag::Latest),
            &[invoke_transaction, invoke_transaction_2],
            [SimulationFlag::SkipFeeCharge],
        )
        .await?;

    assert_eq!(simulations.len(), 2);
    assert_eq!(simulations[0].fee_estimation.gas_consumed, 0);
    assert_eq!(simulations[0].fee_estimation.overall_fee, 210);
    assert_eq!(simulations[0].fee_estimation.gas_price, 0);

    Ok(())
}
