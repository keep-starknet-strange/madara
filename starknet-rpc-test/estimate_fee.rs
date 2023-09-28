extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedTransaction, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::fixtures::{broadcasted_declare_txn_v1, madara};
use starknet_rpc_test::MadaraClient;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(
    #[future] madara: MadaraClient,
    broadcasted_declare_txn_v1: BroadcastedTransaction,
) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc.estimate_fee(&vec![
            broadcasted_declare_txn_v1
        ], BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_one_txn_cannot_be_executed(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.estimate_fee(&vec![
            BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
                max_fee: FieldElement::default(),
                nonce: FieldElement::ZERO,
                sender_address: FieldElement::default(),
                signature: vec![],
                calldata: vec![FieldElement::from_hex_be("0x0").unwrap()],
                is_query: true,
            }),
        ], BlockId::Tag(BlockTag::Latest)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn returns_same_vec_length_as_txns(
    #[future] madara: MadaraClient,
    broadcasted_declare_txn_v1: BroadcastedTransaction,
) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let estimate = rpc
        .estimate_fee(
            &vec![broadcasted_declare_txn_v1.clone(), broadcasted_declare_txn_v1],
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    assert_eq!(estimate.len(), 2);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(
    #[future] madara: MadaraClient,
    broadcasted_declare_txn_v1: BroadcastedTransaction,
) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let estimate = rpc.estimate_fee(&vec![broadcasted_declare_txn_v1], BlockId::Tag(BlockTag::Latest)).await?;

    assert!(estimate[0].overall_fee > 0);
    assert!(estimate[0].gas_consumed > 0);

    Ok(())
}
