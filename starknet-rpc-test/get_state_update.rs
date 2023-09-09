extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, BlockTag, FieldElement, MaybePendingStateUpdate, StarknetError, StateDiff};
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE};
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
        .get_state_update(
            BlockId::Hash(FieldElement::ZERO),
        )
        .await,
        Err(ProviderError::StarknetError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn can_get_state_update(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.run_to_block(5).await?;

    let latest_block = rpc.block_hash_and_number().await?;

    let maybe_state_update = rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap();

    let state_update = match maybe_state_update {
        MaybePendingStateUpdate::Update(value) => value,
        _ => panic!("unexpected data type"),
    };

    assert_eq!(state_update.block_hash, latest_block.block_hash);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn state_diff_is_valid_given_txn(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
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

    let latest_block = rpc.block_hash_and_number().await?;

    let maybe_state_update = rpc.get_state_update(BlockId::Tag(BlockTag::Latest)).await.unwrap();

    let state_update = match maybe_state_update {
        MaybePendingStateUpdate::Update(value) => value,
        _ => panic!("unexpected data type"),
    };

    assert_eq!(state_update.block_hash, latest_block.block_hash);

    // TODO:
    // check with actual values once StateDiffs actually diff
    // see crates/client/rpc/src/lib.rs#get_state_update

    Ok(())
}
