#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::{Provider, ProviderError};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction};

#[rstest]
#[tokio::test]
async fn work_ok_with_empty_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;
    assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.get_block_transaction_count(BlockId::Hash(FieldElement::ZERO)).await.err(),
        Some(ProviderError::StarknetError(_))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_with_block_one_tx(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let token_transfer_tx = account.transfer_tokens(
        account.address(),
        FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
        None,
    );

    madara.create_block_with_txs(vec![Transaction::Execution(token_transfer_tx)]).await?;

    assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 1);

    Ok(())
}

// TODO: Uncomment when raw execution is supported
// #[rstest]
// #[tokio::test]
// async fn work_ok_with_block_multiple_txs(#[future] _madara: MadaraClient) -> Result<(),
// anyhow::Error> {     let madara = madara.await;
//     let rpc = madara.get_starknet_client();

//     madara
//         .create_block_with_txs(
//             vec![
//                 account.transfer_tokens(
//                     argent_account_address,
//                     FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
//                     Some(1),
//                 ),
//                 account.transfer_tokens(
//                     argent_account_address,
//                     FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
//                     Some(2),
//                 ),
//             ],
//         )
//         .await?;

//     assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 2);

//     Ok(())
// }
