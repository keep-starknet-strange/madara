extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, BroadcastedInvokeTransaction, BroadcastedTransaction, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::ACCOUNT_CONTRACT;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let ok_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be("5a02acdbf218464be3dd787df7a302f71fab586cad5588410ba88b3ed7b3a21").unwrap(),
            FieldElement::from_hex_be("3d7905601c217734671143d457f0db37f7f8883112abd34b92c4abfeafde0c3").unwrap(),
            FieldElement::from_hex_be("2").unwrap(),
            FieldElement::from_hex_be("e150b6c2db6ed644483b01685571de46d2045f267d437632b508c19f3eb877").unwrap(),
            FieldElement::from_hex_be("494196e88ce16bff11180d59f3c75e4ba3475d9fba76249ab5f044bcd25add6").unwrap(),
        ],
        is_query: true,
    });

    assert_matches!(
        rpc.estimate_fee(&vec![ok_invoke_transaction], BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_one_txn_cannot_be_executed(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let bad_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::default(),
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::default(),
        signature: vec![],
        calldata: vec![FieldElement::from_hex_be("0x0").unwrap()],
        is_query: true,
    });

    // from mainnet tx: 0x000c52079f33dcb44a58904fac3803fd908ac28d6632b67179ee06f2daccb4b5
    // https://starkscan.co/tx/0x000c52079f33dcb44a58904fac3803fd908ac28d6632b67179ee06f2daccb4b5
    let ok_invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be("5a02acdbf218464be3dd787df7a302f71fab586cad5588410ba88b3ed7b3a21").unwrap(),
            FieldElement::from_hex_be("3d7905601c217734671143d457f0db37f7f8883112abd34b92c4abfeafde0c3").unwrap(),
            FieldElement::from_hex_be("2").unwrap(),
            FieldElement::from_hex_be("e150b6c2db6ed644483b01685571de46d2045f267d437632b508c19f3eb877").unwrap(),
            FieldElement::from_hex_be("494196e88ce16bff11180d59f3c75e4ba3475d9fba76249ab5f044bcd25add6").unwrap(),
        ],
        is_query: true,
    });

    assert_matches!(
        rpc.estimate_fee(&vec![
            bad_invoke_transaction,
            ok_invoke_transaction,
        ], BlockId::Tag(BlockTag::Latest)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // from mainnet tx: 0x000c52079f33dcb44a58904fac3803fd908ac28d6632b67179ee06f2daccb4b5
    // https://starkscan.co/tx/0x000c52079f33dcb44a58904fac3803fd908ac28d6632b67179ee06f2daccb4b5
    let invoke_transaction = BroadcastedTransaction::Invoke(BroadcastedInvokeTransaction {
        max_fee: FieldElement::ZERO,
        signature: vec![],
        nonce: FieldElement::ZERO,
        sender_address: FieldElement::from_hex_be(ACCOUNT_CONTRACT).unwrap(),
        calldata: vec![
            FieldElement::from_hex_be("5a02acdbf218464be3dd787df7a302f71fab586cad5588410ba88b3ed7b3a21").unwrap(),
            FieldElement::from_hex_be("3d7905601c217734671143d457f0db37f7f8883112abd34b92c4abfeafde0c3").unwrap(),
            FieldElement::from_hex_be("2").unwrap(),
            FieldElement::from_hex_be("e150b6c2db6ed644483b01685571de46d2045f267d437632b508c19f3eb877").unwrap(),
            FieldElement::from_hex_be("494196e88ce16bff11180d59f3c75e4ba3475d9fba76249ab5f044bcd25add6").unwrap(),
        ],
        is_query: true,
    });

    let estimate =
        rpc.estimate_fee(&vec![invoke_transaction.clone(), invoke_transaction], BlockId::Tag(BlockTag::Latest)).await?;

    // TODO: instead execute the tx and check that the actual fee are the same as the estimated ones
    assert_eq!(estimate.len(), 2);
    assert_eq!(estimate[0].overall_fee, 410);
    assert_eq!(estimate[1].overall_fee, 410);
    // https://starkscan.co/block/5
    assert_eq!(estimate[0].gas_consumed, 0);
    assert_eq!(estimate[1].gas_consumed, 0);

    Ok(())
}
