extern crate starknet_rpc_test;

use anyhow::anyhow;
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, DeclaredClassItem, MaybePendingStateUpdate, NonceUpdate, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc
        .get_state_update(
            BlockId::Hash(FieldElement::ZERO),
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn returns_correct_state_diff_transfer(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let recipient = FieldElement::from_hex_be("0x1234").unwrap();
    let sender_account_address = FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).expect("Invalid Contract Address");

    let (state_update, block_hash, new_nonce) = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                recipient,
                FieldElement::ONE,
                None,
            ))])
            .await?;

        let state_update = match rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
            MaybePendingStateUpdate::Update(update) => update,
            MaybePendingStateUpdate::PendingUpdate(_) => return Err(anyhow!("Expected update, got pending update")),
        };
        let block_hash_and_number = rpc.block_hash_and_number().await?;
        let new_nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), sender_account_address).await?;

        (state_update, block_hash_and_number.block_hash, new_nonce)
    };

    assert_eq!(state_update.block_hash, block_hash);
    assert_eq!(state_update.old_root, FieldElement::ZERO);
    assert_eq!(state_update.new_root, FieldElement::ZERO);

    assert_eq!(state_update.state_diff.nonces.len(), 1);
    assert_eq!(
        state_update.state_diff.nonces[0],
        NonceUpdate { contract_address: sender_account_address, nonce: new_nonce }
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn returns_correct_state_diff_declare(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash, expected_compiled_class_hash) = account.declare_contract(
        "./contracts/counter6/counter6.contract_class.json",
        "./contracts/counter6/counter6.compiled_contract_class.json",
    );

    let (state_update, block_hash) = {
        let mut madara_write_lock = madara.write().await;

        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

        let state_update = match rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
            MaybePendingStateUpdate::Update(update) => update,
            MaybePendingStateUpdate::PendingUpdate(_) => return Err(anyhow!("Expected update, got pending update")),
        };
        let block_hash_and_number = rpc.block_hash_and_number().await?;

        (state_update, block_hash_and_number.block_hash)
    };

    assert_eq!(state_update.block_hash, block_hash);
    assert_eq!(state_update.old_root, FieldElement::ZERO);
    assert_eq!(state_update.new_root, FieldElement::ZERO);

    assert_eq!(state_update.state_diff.declared_classes.len(), 1);
    assert_eq!(
        state_update.state_diff.declared_classes[0],
        DeclaredClassItem { class_hash: expected_class_hash, compiled_class_hash: expected_compiled_class_hash }
    );

    Ok(())
}
