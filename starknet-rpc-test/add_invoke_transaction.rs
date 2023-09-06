extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Account, AccountFactory, ExecutionEncoding, OpenZeppelinAccountFactory, SingleOwnerAccount};
use starknet_core::chain_id;
use starknet_core::types::{InvokeTransactionResult, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, read_erc20_balance, AccountActions};
use starknet_rpc_test::{MadaraClient, SendTransactionError, Transaction, TransactionResult};
use starknet_signers::{LocalWallet, SigningKey};

#[rstest]
#[tokio::test]
async fn fail_validation_step(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    // using incorrect private key to generate the wrong signature
    let account = create_account(rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);

    let mut txs = madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            FieldElement::from_hex_be("0x123").unwrap(),
            FieldElement::ONE,
            None,
        ))])
        .await?;

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs.remove(0);
    assert_matches!(
        invoke_tx_result.err(),
        Some(SendTransactionError::AccountError(starknet_accounts::AccountError::Provider(
            ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ValidationFailure),
                message: _
            })
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

    let invoke_tx_result = txs.remove(0).unwrap();
    match invoke_tx_result {
        TransactionResult::Execution(InvokeTransactionResult { transaction_hash }) => {
            assert_eq!(
                transaction_hash,
                FieldElement::from_hex_be("0x062ab35d456761550b667f14633d182d250285cac50991f3b0eb24c4c3be6979")
                    .unwrap()
            )
        }
        _ => panic!("Unexpected transaction result"),
    }
    assert_eq!(final_balance[1], initial_balance[1]); // higher 128 bits are equal
    assert_eq!(final_balance[0] - initial_balance[0], FieldElement::ONE); // lower 128 bits differ by one

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_execution_step_with_no_strage_change(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
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

    let mut txs = madara
        .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens_u256(
            recipient_account,
            [funding_account_balance[0], funding_account_balance[1]], // send all the available funds
            None,
        ))])
        .await?;

    let final_balance = read_erc20_balance(rpc, fee_token_address, recipient_account).await;

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs.remove(0);

    assert!(invoke_tx_result.is_ok()); // the transaction was sent successfully
    assert_eq!(final_balance, initial_balance);

    Ok(())
}
