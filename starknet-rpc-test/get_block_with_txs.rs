#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use anyhow::anyhow;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{
    BlockId, BlockStatus, BlockTag, DeclareTransaction, InvokeTransaction, MaybePendingBlockWithTxs, StarknetError,
    Transaction as StarknetTransaction,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    build_deploy_account_tx, build_oz_account_factory, build_single_owner_account, AccountActions,
};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.get_block_with_txs(BlockId::Hash(FieldElement::ZERO)).await.err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_invoke_txn(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let recipient = FieldElement::from_hex_be("0x1234").unwrap();
    let (current_nonce, block) = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        let nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address()).await?;

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                recipient,
                FieldElement::ONE,
                None,
            ))])
            .await?;

        let block = match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
            MaybePendingBlockWithTxs::Block(block) => block,
            MaybePendingBlockWithTxs::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
        };

        (nonce, block)
    };

    assert_eq!(block.transactions.len(), 1);
    let tx = match &block.transactions[0] {
        StarknetTransaction::Invoke(InvokeTransaction::V1(tx)) => tx,
        _ => return Err(anyhow!("Expected an invoke transaction v1")),
    };
    assert_eq!(tx.sender_address, FieldElement::TWO);
    assert_eq!(tx.nonce, current_nonce);
    assert_eq!(tx.max_fee, FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap());
    assert_eq!(
        tx.calldata,
        vec![
            FieldElement::ONE,
            FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
            get_selector_from_name("transfer").unwrap(),
            FieldElement::ZERO,
            FieldElement::THREE,
            FieldElement::THREE,
            recipient,
            FieldElement::ONE,
            FieldElement::ZERO,
        ]
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_deploy_account_txn(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let class_hash = FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap();
    let contract_address_salt = FieldElement::ONE;
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    let block = {
        let mut madara_write_lock = madara.write().await;
        let oz_factory = build_oz_account_factory(&rpc, "0x123", class_hash).await;
        let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);

        let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        let account_address = account_deploy_txn.address();

        // We execute the funding in a different block, because we have no way to guarantee the tx execution
        // order once in the mempool
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
                account_address,
                max_fee,
                None,
            ))])
            .await?;

        madara_write_lock.create_block_with_txs(vec![Transaction::AccountDeployment(account_deploy_txn)]).await?;

        match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
            MaybePendingBlockWithTxs::Block(block) => block,
            MaybePendingBlockWithTxs::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
        }
    };

    assert_eq!(block.transactions.len(), 1);
    let tx = match &block.transactions[0] {
        StarknetTransaction::DeployAccount(tx) => tx,
        _ => return Err(anyhow!("Expected an deploy transaction v1")),
    };
    assert_eq!(tx.nonce, 0u8.into());
    assert_eq!(tx.max_fee, max_fee);
    assert_eq!(tx.contract_address_salt, contract_address_salt);
    assert_eq!(tx.class_hash, class_hash);
    assert_eq!(
        tx.constructor_calldata,
        vec![FieldElement::from_hex_be("0x0566d69d8c99f62bc71118399bab25c1f03719463eab8d6a444cd11ece131616").unwrap(),]
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_declare_txn(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // manually setting fee else estimate_fee will be called and it will fail
    // as the nonce has not been updated yet
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    let (current_nonce, class_hash, compiled_class_hash, block) = {
        let mut madara_write_lock = madara.write().await;

        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        let nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address()).await?;
        let (declare_tx, class_hash, compiled_class_hash) = account.declare_contract(
            "./contracts/counter5/counter5.contract_class.json",
            "./contracts/counter5/counter5.compiled_contract_class.json",
        );

        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

        let block = match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
            MaybePendingBlockWithTxs::Block(block) => block,
            MaybePendingBlockWithTxs::PendingBlock(_) => {
                return Err(anyhow!("Expected block, got pending block"));
            }
        };
        (nonce, class_hash, compiled_class_hash, block)
    };

    assert_eq!(block.status, BlockStatus::AcceptedOnL2);
    assert_eq!(block.transactions.len(), 1);
    let tx = match &block.transactions[0] {
        StarknetTransaction::Declare(DeclareTransaction::V2(tx)) => tx,
        _ => return Err(anyhow!("Expected an declare transaction v2")),
    };
    assert_eq!(tx.sender_address, FieldElement::TWO);
    assert_eq!(tx.nonce, current_nonce);
    assert_eq!(tx.max_fee, max_fee);
    assert_eq!(tx.class_hash, class_hash);
    assert_eq!(tx.compiled_class_hash, compiled_class_hash);

    Ok(())
}
