extern crate starknet_rpc_test;
use std::time::Duration;
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Call, ConnectedAccount};
use starknet_core::types::{
    BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedTransaction, MaybePendingTransactionReceipt,
    PendingTransactionReceipt, StarknetError, TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ACCOUNT_CONTRACT, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use tokio::time;
use starknet_test_utils::constants::{ACCOUNT_CONTRACT, TEST_CONTRACT_ADDRESS};
use starknet_test_utils::fixtures::{madara, ThreadSafeMadaraClient};

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
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let nonce = funding_account.get_nonce().await?;
    let max_fee = FieldElement::from(1000u16);
    let calls = vec![Call {
        to: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        selector: get_selector_from_name("sqrt").unwrap(),
        calldata: vec![FieldElement::from(81u8)],
    }];

    let tx = funding_account.prepare_invoke(calls, nonce, max_fee, false).await;

    let invoke_transaction = BroadcastedTransaction::Invoke(tx.clone());

    let estimates = rpc.estimate_fee(&vec![invoke_transaction], BlockId::Tag(BlockTag::Latest)).await?;

    let invoke_transaction_result = rpc.add_invoke_transaction(tx.clone()).await?;

    const SLEEP_DURATION: u64 = 20; // Sleep duration in seconds

    loop {
        let invoke_tx_receipt = rpc.get_transaction_receipt(invoke_transaction_result.transaction_hash).await?;

        match invoke_tx_receipt {
            MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt)) => {
                assert_eq!(FieldElement::from(estimates[0].overall_fee), receipt.actual_fee);
                break; // Break the loop if receipt is received
            }

            MaybePendingTransactionReceipt::PendingReceipt(PendingTransactionReceipt::Invoke(_)) => {
                time::sleep(Duration::from_secs(SLEEP_DURATION)).await;
            }

            _ => {
                panic!("expected invoke transaction receipt");
            }
        }
    }

    assert_eq!(estimates.len(), 1);
    assert_eq!(estimates[0].gas_consumed, 0);

    Ok(())
}
