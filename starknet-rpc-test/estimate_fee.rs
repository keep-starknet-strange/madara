extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedTransaction, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ACCOUNT_CONTRACT, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

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
        is_query: true,
    });

    assert_matches!(
        rpc.estimate_fee(&vec![ok_invoke_transaction], BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
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
        is_query: true,
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
        is_query: true,
    });

    assert_matches!(
        rpc.estimate_fee(&vec![
            bad_invoke_transaction,
            ok_invoke_transaction,
        ], BlockId::Tag(BlockTag::Latest)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let tx = BroadcastedInvokeTransaction {
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
        is_query: true,
    };

    let invoke_transaction = BroadcastedTransaction::Invoke(tx.clone());

    let invoke_transaction_2 =
        BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction { nonce: FieldElement::ONE, ..tx });

    let estimates =
        rpc.estimate_fee(&vec![invoke_transaction, invoke_transaction_2], BlockId::Tag(BlockTag::Latest)).await?;

    // TODO: instead execute the tx and check that the actual fee are the same as the estimated ones
    assert_eq!(estimates.len(), 2);
    assert_eq!(estimates[0].overall_fee, 210);
    assert_eq!(estimates[1].overall_fee, 210);
    // https://starkscan.co/block/5
    assert_eq!(estimates[0].gas_consumed, 0);
    assert_eq!(estimates[1].gas_consumed, 0);

    Ok(())
}
