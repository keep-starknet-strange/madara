use std::vec;

use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::BlockId;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    build_single_owner_account, is_good_error_code, read_erc20_balance, AccountActions, U256,
};
use starknet_rpc_test::{SendTransactionError, Transaction};
use starknet_test_utils::constants::ETH_FEE_TOKEN_ADDRESS;

#[rstest]
#[tokio::test]
async fn fail_validation_step(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let txs = {
        let mut madara_write_lock = madara.write().await;
        // using incorrect private key to generate the wrong signature
        let account = build_single_owner_account(&rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                FieldElement::from_hex_be("0x123").unwrap(),
                FieldElement::ONE,
                None,
            ))])
            .await?
    };

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs[0].as_ref().unwrap_err();
    match invoke_tx_result {
        SendTransactionError::AccountError(starknet_accounts::AccountError::Provider(provider_error)) => {
            assert!(is_good_error_code(provider_error, 55));
        }
        _ => {
            panic!("wrong error type");
        }
    };
    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_storage_change(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let recipient_account = FieldElement::from_hex_be("0x123").unwrap();

    let fee_token_address = FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap();
    let (txs, recipient_initial_balance, recipient_final_balance, block_number) = {
        let mut madara_write_lock = madara.write().await;
        let initial_balance = read_erc20_balance(&rpc, fee_token_address, recipient_account).await;

        let txs = madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
                recipient_account,
                FieldElement::ONE,
                None,
            ))])
            .await?;

        let final_balance = read_erc20_balance(&rpc, fee_token_address, recipient_account).await;
        let block_number = rpc.block_number().await?;
        (txs, initial_balance, final_balance, block_number)
    };

    assert_eq!(txs.len(), 1);

    assert!(txs[0].is_ok());
    assert_eq!(recipient_final_balance[1], recipient_initial_balance[1]); // higher 128 bits are equal
    assert_eq!(recipient_final_balance[0] - recipient_initial_balance[0], FieldElement::ONE); // lower 128 bits differ by one

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_execution_revert_with_no_transfer(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    // we will try to transfer all the funds of the funding account
    // so the transaction will fail in the execution step as we won't have
    // funds to pay the fees

    let rpc = madara.get_starknet_client().await;

    let fee_token_address = FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap();

    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let (block_number, recipient_initial_balance, recipient_final_balance, txs) = {
        let mut madara_write_lock = madara.write().await;
        let funding_account_balance = read_erc20_balance(&rpc, fee_token_address, funding_account.address()).await;

        let recipient_account = FieldElement::from_hex_be("0x123").unwrap();
        let initial_balance = read_erc20_balance(&rpc, fee_token_address, recipient_account).await;

        let txs = madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens_u256(
                recipient_account,
                U256 { low: funding_account_balance[0], high: funding_account_balance[1] }, /* send all the
                                                                                             * available funds */
                None,
            ))])
            .await?;

        let final_balance = read_erc20_balance(&rpc, fee_token_address, recipient_account).await;
        let block_number = rpc.block_number().await?;

        (block_number, initial_balance, final_balance, txs)
    };

    assert_eq!(txs.len(), 1);

    let invoke_tx_result = txs[0].as_ref();

    assert!(invoke_tx_result.is_ok()); // the transaction was sent successfully
    assert_eq!(recipient_final_balance, recipient_initial_balance);

    // get included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1);

    Ok(())
}
