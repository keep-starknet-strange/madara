#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use anyhow::anyhow;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, MaybePendingBlockWithTxHashes, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.get_block_with_tx_hashes(BlockId::Hash(FieldElement::ZERO)).await.err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_correct_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let block = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                FieldElement::from_hex_be("0x1234").unwrap(),
                FieldElement::ONE,
                None,
            ))])
            .await?;

        match rpc.get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest)).await? {
            MaybePendingBlockWithTxHashes::Block(block) => block,
            MaybePendingBlockWithTxHashes::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
        }
    };

    assert_eq!(block.transactions.len(), 1);

    Ok(())
}
