#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, BlockTag, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE, TEST_CONTRACT_ADDRESS,
};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc
        .get_nonce(
            BlockId::Hash(FieldElement::ZERO),
            FieldElement::from_hex_be(CONTRACT_ADDRESS).expect("Invalid Contract Address"),
        )
        .await,
        Err(ProviderError::StarknetError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_non_used_contract_address(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(
        rpc.get_nonce(
            BlockId::Number(0),
            FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract Address")
        )
        .await
        .ok(),
        Some(FieldElement::ZERO)
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_non_account_contract(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(
        rpc.get_nonce(
            BlockId::Number(0),
            FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).expect("Invalid Contract Address")
        )
        .await
        .ok(),
        Some(FieldElement::ZERO)
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_account_with_tx(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            account.address(),
            FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
            None,
        ))])
        .await?;

    assert_eq!(rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address(),).await.ok(), Some(FieldElement::ONE));

    Ok(())
}
