extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};

#[rstest]
#[tokio::test]
async fn works_with_one_pending_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let mut madara_write_lock = madara.write().await;
    account.transfer_tokens(FieldElement::from_hex_be("0x123").unwrap(), FieldElement::ONE, None).send().await?;

    let pending_txs = rpc.pending_transactions().await?;

    // Seal block
    madara_write_lock.create_empty_block().await?;

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_50_pending_transactions(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let mut madara_write_lock = madara.write().await;
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address()).await?;
    let nonce = nonce.to_bytes_be();
    let nonce: u64 = nonce[31] as u64;

    // loop from 0 to 50
    for nonce_idx in 0..50 {
        let _ = account
            .transfer_tokens(
                FieldElement::from_hex_be("0x123").unwrap(),
                FieldElement::ONE,
                Some(nonce + nonce_idx as u64),
            )
            .send()
            .await;
    }

    let pending_txs = rpc.pending_transactions().await?;
    // Seal block
    madara_write_lock.create_empty_block().await?;

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 50);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_without_pending_transactions(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let pending_txs = {
        let _madara_write_lock = madara.write();
        rpc.pending_transactions().await?
    };

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 0);

    Ok(())
}
