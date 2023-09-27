extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{FEE_TOKEN_ADDRESS, MAX_U256};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).expect("Invalid Contract Address");

    assert_matches!(
        rpc
        .get_storage_at(
            fee_token_address,
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Hash(FieldElement::ZERO),
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_contract(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let invalid_contract_address =
        FieldElement::from_hex_be("0x051e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00")
            .expect("Invalid Contract Address");

    assert_matches!(rpc
        .get_storage_at(
            invalid_contract_address,
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Number(0),
        )
        .await,
        Err(StarknetProviderError(
            StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(code),
                ..
            }
        )) if code == StarknetError::ContractNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_at_previous_contract(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).expect("Invalid Contract Address");

    assert_eq!(
        rpc.get_storage_at(
            fee_token_address,
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Number(0)
        )
        .await?,
        FieldElement::from_hex_be(MAX_U256).unwrap()
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn return_0_for_uninitialized_key(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).expect("Invalid Contract Address");

    assert_eq!(
        rpc.get_storage_at(fee_token_address, FieldElement::from_hex_be("0x1").unwrap(), BlockId::Number(0),).await?,
        FieldElement::ZERO
    );

    Ok(())
}
