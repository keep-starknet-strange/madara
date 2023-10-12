extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, DeclareTransactionResult, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, read_erc20_balance, AccountActions, U256};
use starknet_rpc_test::{SendTransactionError, Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn fail_validation_step(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let txs = {
        // using incorrect private key to generate the wrong signature
        let account = build_single_owner_account(&rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);
        let (declare_tx, _, _) =
            account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?
    };
    assert_eq!(txs.len(), 1);

    let declare_tx_result = txs[0].as_ref().unwrap_err();
    assert_matches!(
        declare_tx_result,
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
#[ignore = "this test drain the account, wich make other tests fail afterward. We have to find another way to make \
            this one fail"]
async fn fail_execution_step_with_no_storage_change(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash, _) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

    let (block_number, txs) = {
        let mut madara_write_lock = madara.write().await;
        // draining account so the txn fails during execution
        let balance =
            read_erc20_balance(&rpc, FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(), account.address()).await;
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens_u256(
                FieldElement::from_hex_be("0x1234").unwrap(),
                // subtractin 150k to keep some fees for the transfer
                U256 { low: balance[0] - FieldElement::from_dec_str("150000").unwrap(), high: balance[1] },
                None,
            ))])
            .await?;

        // declaring contract
        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;
        let block_number = rpc.block_number().await?;
        (block_number, txs)
    };
    assert_eq!(txs.len(), 1);
    assert!(txs[0].is_ok());

    // transaction failed during execution, no change in storage
    assert!(rpc.get_class(BlockId::Number(block_number), expected_class_hash).await.is_err());

    // doesn't get included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "class already declared"]
async fn works_with_storage_change(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash, _) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

    let (mut txs, block_number) = {
        let mut madara_write_lock = madara.write().await;
        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;
        let block_number = rpc.block_number().await?;
        (txs, block_number)
    };

    assert_eq!(txs.len(), 1);
    let declare_tx_result = txs.remove(0);
    match declare_tx_result {
        Ok(TransactionResult::Declaration(DeclareTransactionResult { transaction_hash, class_hash })) => {
            assert_eq!(
                transaction_hash,
                FieldElement::from_hex_be("0x05e0f64e8140019f2657f244dd9fd136d18acc6f52d8a0b85d3f84a110d4c708")
                    .unwrap()
            );
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
#[ignore = "unpredictable behaviour depending on the test execution order"]
async fn fails_already_declared(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // first declaration works
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, _, _) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

    let mut madara_write_lock = madara.write().await;
    // The first one will fail too for now
    let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

    assert_eq!(txs.len(), 1);
    assert!(txs[0].as_ref().is_ok());

    // second declaration fails
    let (declare_tx, _, _) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

    let mut txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

    assert_eq!(txs.len(), 1);
    let declare_tx_result = txs.remove(0);
    assert_matches!(
        declare_tx_result.err(),
        Some(SendTransactionError::AccountError(starknet_accounts::AccountError::Provider(
            ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ClassAlreadyDeclared),
                message: _
            })
        )))
    );

    Ok(())
}
