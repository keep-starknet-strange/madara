extern crate starknet_rpc_test;

use core::panic;
use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, DeclareTransactionResult, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{Provider, ProviderError};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, OZ_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    build_single_owner_account, is_good_error_code, read_erc20_balance, AccountActions, U256,
};
use starknet_rpc_test::{SendTransactionError, Transaction, TransactionResult};
use starknet_test_utils::constants::ETH_FEE_TOKEN_ADDRESS;

#[rstest]
#[tokio::test]
async fn fail_validation_step(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let txs = {
        // using incorrect private key to generate the wrong signature
        let account = build_single_owner_account(&rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);
        let (declare_tx, _, _) = account.declare_contract(
            "../starknet-rpc-test/contracts/counter0/counter0.contract_class.json",
            "../starknet-rpc-test/contracts/counter0/counter0.compiled_contract_class.json",
            None,
        );

        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?
    };
    assert_eq!(txs.len(), 1);

    let declare_tx_err = txs[0].as_ref().unwrap_err();
    match declare_tx_err {
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
async fn fail_execution_step_with_no_storage_change(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let oz_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, OZ_CONTRACT_ADDRESS, true);

    let (block_number, expected_class_hash) = {
        let mut madara_write_lock = madara.write().await;

        let block_number = rpc.block_number().await?;
        let current_nonce = rpc
            .get_nonce(BlockId::Number(block_number), FieldElement::from_hex_be(OZ_CONTRACT_ADDRESS).unwrap())
            .await?;

        let (declare_tx, expected_class_hash, _) = oz_account.declare_contract(
            "../starknet-rpc-test/contracts/counter1/counter1.contract_class.json",
            "../starknet-rpc-test/contracts/counter1/counter1.compiled_contract_class.json",
            Some(current_nonce + FieldElement::ONE),
        );
        // draining oz_account so the txn fails during execution
        let balance =
            read_erc20_balance(&rpc, FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap(), oz_account.address())
                .await;
        let txs = madara_write_lock
            .create_block_with_txs(vec![
                Transaction::Execution(oz_account.transfer_tokens_u256(
                    FieldElement::from_hex_be("0x123").unwrap(),
                    // subtractin 150k to keep some fees for the transfer
                    // but not enough for the declare
                    U256 { low: balance[0] - FieldElement::from(150_000u128), high: balance[1] },
                    None,
                )),
                Transaction::Declaration(declare_tx),
            ])
            .await?;
        // Both tx made it into the mempool
        assert!(txs[0].is_ok());
        assert!(txs[1].is_ok());

        (rpc.block_number().await?, expected_class_hash)
    };

    // transaction failed during execution, no change in storage
    assert!(rpc.get_class(BlockId::Number(block_number), expected_class_hash).await.is_err());

    // doesn't get included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_storage_change(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash, _) = account.declare_contract(
        "../starknet-rpc-test/contracts/counter2/counter2.contract_class.json",
        "../starknet-rpc-test/contracts/counter2/counter2.compiled_contract_class.json",
        None,
    );

    let (mut txs, block_number) = {
        let mut madara_write_lock = madara.write().await;
        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;
        let block_number = rpc.block_number().await?;
        (txs, block_number)
    };

    assert_eq!(txs.len(), 1);
    let declare_tx_result = txs.remove(0);
    match declare_tx_result {
        Ok(TransactionResult::Declaration(DeclareTransactionResult { transaction_hash: _, class_hash })) => {
            assert_eq!(class_hash, expected_class_hash);
        }
        _ => panic!("Expected declare transaction result"),
    }

    assert!(rpc.get_class(BlockId::Number(block_number), expected_class_hash).await.is_ok());

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fails_already_declared(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // first declaration works
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, _, _) = account.declare_contract(
        "../starknet-rpc-test/contracts/counter3/counter3.contract_class.json",
        "../starknet-rpc-test/contracts/counter3/counter3.compiled_contract_class.json",
        None,
    );

    let mut madara_write_lock = madara.write().await;
    // The first one will fail too for now
    let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

    assert_eq!(txs.len(), 1);
    assert!(txs[0].as_ref().is_ok());

    // second declaration fails
    let (declare_tx, _, _) = account.declare_contract(
        "../starknet-rpc-test/contracts/counter3/counter3.contract_class.json",
        "../starknet-rpc-test/contracts/counter3/counter3.compiled_contract_class.json",
        None,
    );

    let mut txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

    assert_eq!(txs.len(), 1);
    let declare_tx_result = txs.remove(0);
    assert_matches!(
        declare_tx_result.err(),
        Some(SendTransactionError::AccountError(starknet_accounts::AccountError::Provider(
            ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared)
        )))
    );

    Ok(())
}
