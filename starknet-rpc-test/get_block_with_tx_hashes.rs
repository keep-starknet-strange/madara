#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use anyhow::anyhow;
use rstest::rstest;
use starknet_core::types::{
    BlockId, BlockStatus, BlockTag, BlockWithTxHashes, MaybePendingBlockWithTxHashes, StarknetError,
};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{assert_equal_blocks_with_tx_hashes, create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

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
async fn works_with_correct_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            FieldElement::from_hex_be("0x1234").unwrap(),
            FieldElement::ONE,
            None,
        ))])
        .await?;

    let block = match rpc.get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingBlockWithTxHashes::Block(block) => block,
        MaybePendingBlockWithTxHashes::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
    };

    assert_equal_blocks_with_tx_hashes(
        block.clone(),
        BlockWithTxHashes {
            status: BlockStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x015e8bc7066c6d98d71c52bd52bb8eb0d1747eaa189c7f90a2a31045edccf2a8")
                .unwrap(),
            parent_hash: FieldElement::from_hex_be(
                "0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad",
            )
            .unwrap(),
            block_number: 1,
            new_root: FieldElement::ZERO,
            sequencer_address: FieldElement::from_hex_be(
                "0x000000000000000000000000000000000000000000000000000000000000dead",
            )
            .unwrap(),
            transactions: vec![
                FieldElement::from_hex_be("0x069d9d0ac1f5a4ad8d8e9a3954da53b5dc8ed239c02ad04492b9e15c50fe6d11")
                    .unwrap(),
            ],
            timestamp: block.timestamp, // timestamps can vary so just using the actual timestamp
        },
    );

    Ok(())
}
