extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, BlockTag, InvokeTransactionResult, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, read_erc20_balance, AccountActions, U256};
use starknet_rpc_test::{MadaraClient, SendTransactionError, Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn fail_validation_step(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    // using incorrect private key to generate the wrong signature
    let account = create_account(rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);

    let txs = madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            FieldElement::from_hex_be("0x123").unwrap(),
            FieldElement::ONE,
            None,
        ))])
        .await?;

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs[0].as_ref().unwrap_err();
    assert_matches!(
        invoke_tx_result,
        SendTransactionError::AccountError(starknet_accounts::AccountError::Provider(ProviderError::StarknetError(
            StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ValidationFailure),
                message: _
            }
        )))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_storage_change(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let recipient_account = FieldElement::from_hex_be("0x123").unwrap();

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let initial_balance = read_erc20_balance(rpc, fee_token_address, recipient_account).await;

    let mut txs = madara
        .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
            recipient_account,
            FieldElement::ONE,
            None,
        ))])
        .await?;

    let final_balance = read_erc20_balance(rpc, fee_token_address, recipient_account).await;

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs.remove(0);
    match invoke_tx_result {
        Ok(TransactionResult::Execution(InvokeTransactionResult { transaction_hash })) => {
            assert_eq!(
                transaction_hash,
                FieldElement::from_hex_be("0x05605a03e0e1ed95469d887a172346ba0ff90a9b25a02214ade7caa978ab3eec")
                    .unwrap()
            )
        }
        _ => panic!("Expected invoke transaction result"),
    }
    assert_eq!(final_balance[1], initial_balance[1]); // higher 128 bits are equal
    assert_eq!(final_balance[0] - initial_balance[0], FieldElement::ONE); // lower 128 bits differ by one

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?;
    assert_eq!(included_txs, 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_execution_step_with_no_storage_change(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    // we will try to transfer all the funds of the funding account
    // so the transaction will fail in the execution step as we won't have
    // funds to pay the fees

    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let funding_account_balance = read_erc20_balance(rpc, fee_token_address, funding_account.address()).await;

    let recipient_account = FieldElement::from_hex_be("0x123").unwrap();
    let initial_balance = read_erc20_balance(rpc, fee_token_address, recipient_account).await;

    let txs = madara
        .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens_u256(
            recipient_account,
            U256 { low: funding_account_balance[0], high: funding_account_balance[1] }, // send all the available funds
            None,
        ))])
        .await?;

    let final_balance = read_erc20_balance(rpc, fee_token_address, recipient_account).await;

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs[0].as_ref();

    assert!(invoke_tx_result.is_ok()); // the transaction was sent successfully
    assert_eq!(final_balance, initial_balance);

    // doesn't get included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?;
    assert_eq!(included_txs, 0);

    Ok(())
}
