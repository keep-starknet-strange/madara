extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{AccountFactory, ExecutionEncoding, OpenZeppelinAccountFactory, SingleOwnerAccount};
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
async fn fail_execution_step(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    // we will transfer 1 wei from the funding account to recipient_one
    // then we will transfer 1 wei from recipient_one to recipient_two
    // since recipient_one won't have funds to pay the fees, the transaction will fail

    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    // deplpoying recipient_one
    let class_hash = FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be("0x123").unwrap()));
    let oz_factory = OpenZeppelinAccountFactory::new(class_hash, chain_id::TESTNET, signer.clone(), rpc).await.unwrap();
    let max_fee: FieldElement = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();
    let account_deploy_txn = oz_factory.deploy(FieldElement::ONE).max_fee(max_fee);
    let recipeint_one = account_deploy_txn.address();
    madara
        .create_block_with_txs(vec![
            Transaction::Execution(funding_account.transfer_tokens(recipeint_one, max_fee, None)),
            Transaction::AccountDeployment(account_deploy_txn),
        ])
        .await?;
    let recipient_one_account =
        SingleOwnerAccount::new(rpc, signer, recipeint_one, chain_id::TESTNET, ExecutionEncoding::New);

    let recipient_two = FieldElement::from_hex_be("0x456").unwrap();
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    // sending funds to recipient_one (will pass)
    madara
        .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
            recipeint_one,
            FieldElement::ONE,
            None,
        ))])
        .await?;

    let initial_balance_recipient_two = read_erc20_balance(rpc, fee_token_address, recipient_two).await;
    // sending funds to recipient_one (should fail)
    let mut txs = madara
        .create_block_with_txs(vec![Transaction::Execution(recipient_one_account.transfer_tokens(
            recipient_two,
            FieldElement::ONE,
            None,
        ))])
        .await?;

    let final_balance_recipient_two = read_erc20_balance(rpc, fee_token_address, recipient_two).await;

    assert_eq!(txs.len(), 1);
    assert!(txs.remove(0).is_err());
    assert_eq!(final_balance_recipient_two, initial_balance_recipient_two); // recipient_two balance doesn't change

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
