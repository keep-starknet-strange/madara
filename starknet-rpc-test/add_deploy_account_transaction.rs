extern crate starknet_rpc_test;

use std::vec;

use rstest::rstest;
use starknet_accounts::AccountFactory;
use starknet_core::types::{BlockId, BlockTag, DeployAccountTransactionResult};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{build_deploy_account_tx, build_oz_account_factory, create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn fail_execution_step_with_no_storage_change(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    // deploy account
    let oz_factory = build_oz_account_factory(
        rpc,
        SIGNER_PRIVATE,
        FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap(),
    )
    .await;
    let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);
    let account_address = account_deploy_txn.address();

    // as the account isn't funded, this should fail
    let txs = madara.create_block_with_txs(vec![Transaction::AccountDeployment(account_deploy_txn)]).await?;

    assert_eq!(txs.len(), 1);
    assert!(txs[0].as_ref().is_ok());

    // transaction fails, nothing at class hash
    assert!(rpc.get_class_hash_at(BlockId::Tag(BlockTag::Latest), account_address).await.is_err());

    // doesn't get included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?;
    assert_eq!(included_txs, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_storage_change(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    // deploy account
    let oz_factory =
        build_oz_account_factory(rpc, "0x123", FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap())
            .await;
    let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);
    let account_address = account_deploy_txn.address();

    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let mut txs = madara
        .create_block_with_txs(vec![
            Transaction::Execution(funding_account.transfer_tokens(
                account_address,
                FieldElement::from_hex_be("0xFFFFFFFFFFFF").unwrap(),
                None,
            )),
            Transaction::AccountDeployment(account_deploy_txn),
        ])
        .await?;

    assert_eq!(txs.len(), 2);
    let account_deploy_tx_result = txs.remove(1);
    match account_deploy_tx_result {
        // passes the validation stage
        Ok(TransactionResult::AccountDeployment(DeployAccountTransactionResult {
            transaction_hash,
            contract_address,
        })) => {
            assert_eq!(
                transaction_hash,
                FieldElement::from_hex_be("0x02105f08ba02511ccef6ff6676a1481645ec33c9e0d9f7d654b0590aa6afb013")
                    .unwrap()
            );
            assert_eq!(contract_address, account_address);
        }
        _ => panic!("Expected declare transaction result"),
    }
    let class_hash_result = rpc.get_class_hash_at(BlockId::Tag(BlockTag::Latest), account_address).await;
    match class_hash_result {
        Ok(class_hash) => assert_eq!(class_hash, oz_factory.class_hash()),
        Err(e) => panic!("Expected class hash to be present, got error: {}", e),
    }

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?;
    assert_eq!(included_txs, 2); // fund transfer + deploy

    Ok(())
}
