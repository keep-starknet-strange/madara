use std::collections::HashMap;

use anyhow::anyhow;
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Account, ConnectedAccount};
use starknet_core::types::{BlockId, BlockTag, DeclaredClassItem, MaybePendingStateUpdate, StarknetError};
use starknet_core::utils::get_storage_var_address;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::constants::{
    ACCOUNT_CONTRACT_ADDRESS, ARGENT_CONTRACT_ADDRESS, OZ_CONTRACT_ADDRESS, SEQUENCER_CONTRACT_ADDRESS, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, read_erc20_balance, AccountActions};
use starknet_rpc_test::Transaction;
use starknet_test_utils::constants::ETH_FEE_TOKEN_ADDRESS;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.get_state_update(BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetError::BlockNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn returns_correct_state_diff_transfer(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let recipient = FieldElement::from_hex_be(ACCOUNT_CONTRACT_ADDRESS).unwrap();
    let fee_token_address = FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap();
    let account_alice = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let account_bob = build_single_owner_account(&rpc, SIGNER_PRIVATE, OZ_CONTRACT_ADDRESS, true);

    let nonce = account_alice.get_nonce().await?.try_into()?;
    {
        let mut madara_write_lock = madara.write().await;
        let txs = madara_write_lock
            .create_block_with_txs(vec![
                Transaction::Execution(account_alice.transfer_tokens(recipient, FieldElement::ONE, Some(nonce))),
                Transaction::Execution(account_bob.transfer_tokens(recipient, FieldElement::ONE, None)),
            ])
            .await?;
        txs.iter().for_each(|tx| assert!(tx.is_ok()));
    }

    let state_update = match rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingStateUpdate::Update(update) => update,
        MaybePendingStateUpdate::PendingUpdate(_) => {
            return Err(anyhow!("Expected update, got pending update"));
        }
    };
    let block_hash_and_number = rpc.block_hash_and_number().await?;

    assert_eq!(state_update.block_hash, block_hash_and_number.block_hash);
    assert_eq!(state_update.old_root, FieldElement::ZERO);
    assert_eq!(state_update.new_root, FieldElement::ZERO);

    let storage_diff = &state_update.state_diff.storage_diffs[0];
    let mut storage_diff_map: HashMap<&FieldElement, &FieldElement> = HashMap::from_iter(
        storage_diff
            .storage_entries
            .iter()
            .map(|item| (&item.key, &item.value))
            .collect::<Vec<(&FieldElement, &FieldElement)>>(),
    );
    for account_address in
        [account_alice.address(), account_bob.address(), FieldElement::from_hex_be(SEQUENCER_CONTRACT_ADDRESS).unwrap()]
    {
        let balance = read_erc20_balance(&rpc, fee_token_address, account_address).await[0]; // omit the second part since it probably won't change
        let key = get_storage_var_address("ERC20_balances", &[account_address]).unwrap();
        assert_eq!(storage_diff_map.remove(&key).unwrap(), &balance);
    }
    assert!(storage_diff_map.is_empty());
    assert_eq!(state_update.state_diff.nonces.len(), 2);
    let mut nonce_map: HashMap<&FieldElement, &FieldElement> = HashMap::from_iter(
        state_update
            .state_diff
            .nonces
            .iter()
            .map(|item| (&item.contract_address, &item.nonce))
            .collect::<Vec<(&FieldElement, &FieldElement)>>(),
    );

    assert_eq!(storage_diff.address, FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap());
    for account_address in [account_alice.address(), account_bob.address()] {
        let account_new_nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account_address).await?;
        assert_eq!(*nonce_map.remove(&account_address).unwrap(), account_new_nonce);
    }
    assert!(nonce_map.is_empty());

    Ok(())
}

#[rstest]
#[tokio::test]
async fn returns_correct_state_diff_declare(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash, expected_compiled_class_hash) = account.declare_contract(
        "../starknet-rpc-test/contracts/counter6/counter6.contract_class.json",
        "../starknet-rpc-test/contracts/counter6/counter6.compiled_contract_class.json",
        None,
    );

    {
        let mut madara_write_lock = madara.write().await;

        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;
        assert!(txs[0].is_ok());
    };
    let state_update = match rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingStateUpdate::Update(update) => update,
        MaybePendingStateUpdate::PendingUpdate(_) => {
            return Err(anyhow!("Expected update, got pending update"));
        }
    };
    let block_hash = rpc.block_hash_and_number().await?.block_hash;

    assert_eq!(state_update.block_hash, block_hash);
    assert_eq!(state_update.old_root, FieldElement::ZERO);
    assert_eq!(state_update.new_root, FieldElement::ZERO);
    assert_eq!(state_update.state_diff.declared_classes.len(), 1);
    assert_eq!(
        state_update.state_diff.declared_classes[0],
        DeclaredClassItem { class_hash: expected_class_hash, compiled_class_hash: expected_compiled_class_hash }
    );

    assert_eq!(state_update.state_diff.nonces.len(), 1);
    assert_eq!(
        state_update.state_diff.storage_diffs[0].address,
        FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap()
    );
    let account_new_nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address()).await?;
    assert_eq!(state_update.state_diff.nonces[0].nonce, account_new_nonce);
    assert_eq!(state_update.state_diff.nonces[0].contract_address, account.address());

    let storage_diff = &state_update.state_diff.storage_diffs[0];
    let mut storage_diff_map: HashMap<&FieldElement, &FieldElement> = HashMap::from_iter(
        storage_diff
            .storage_entries
            .iter()
            .map(|item| (&item.key, &item.value))
            .collect::<Vec<(&FieldElement, &FieldElement)>>(),
    );
    let fee_token_address = FieldElement::from_hex_be(ETH_FEE_TOKEN_ADDRESS).unwrap();
    for account_address in [account.address(), FieldElement::from_hex_be(SEQUENCER_CONTRACT_ADDRESS).unwrap()] {
        let balance = read_erc20_balance(&rpc, fee_token_address, account_address).await[0]; // omit the second part since it probably won't change
        let key = get_storage_var_address("ERC20_balances", &[account_address]).unwrap();
        assert_eq!(storage_diff_map.remove(&key).unwrap(), &balance);
    }
    assert!(storage_diff_map.is_empty());

    Ok(())
}
