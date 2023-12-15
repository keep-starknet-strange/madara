extern crate starknet_rpc_test;

use std::vec;

use rstest::rstest;
use starknet_accounts::AccountFactory;
use starknet_core::types::{BlockId, BlockTag, DeployAccountTransactionResult};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{
    ARGENT_CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
    MAX_FEE_OVERRIDE, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    build_argent_account_factory, build_deploy_account_tx, build_oz_account_factory, build_single_owner_account,
    AccountActions,
};
use starknet_rpc_test::{Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn argent_account_deployment_and_transfer_works(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // deploy account
    let private_key = "0x789";
    let argent_factory = build_argent_account_factory(
        &rpc,
        private_key,
        FieldElement::from_hex_be(ARGENT_CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap(),
    )
    .await;
    let account_deploy_txn = build_deploy_account_tx(&argent_factory, FieldElement::ONE);
    let account_address = account_deploy_txn.address();

    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let (mut txs, block_number) = {
        let mut madara_write_lock = madara.write().await;
        let argent_account = build_single_owner_account(&rpc, private_key, account_address.into(), true);
        let mut txs = madara_write_lock
            .create_block_with_txs(vec![
                Transaction::Execution(funding_account.transfer_tokens(
                    account_address,
                    FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                    None,
                )),
                Transaction::AccountDeployment(account_deploy_txn),
                Transaction::Execution(argent_account.transfer_tokens(
                    account_address,
                    FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                    None,
                )),
            ])
            .await?;
        let block_number = rpc.block_number().await?;
        (txs, block_number)
    };
    assert_eq!(txs.len(), 3);
    let account_deploy_tx_result = txs.remove(1);
    match account_deploy_tx_result {
        // passes the validation stage
        Ok(TransactionResult::AccountDeployment(DeployAccountTransactionResult {
            transaction_hash: _,
            contract_address,
        })) => {
            assert_eq!(contract_address, account_address);
        }
        _ => panic!("Expected deploy transaction result"),
    }
    let class_hash_result = rpc.get_class_hash_at(BlockId::Number(block_number), account_address).await;
    match class_hash_result {
        Ok(class_hash) => assert_eq!(class_hash, argent_factory.class_hash()),
        Err(e) => panic!("Expected class hash to be present, got error: {}", e),
    }

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1); // Decomposed into 2 blocks

    Ok(())
}
