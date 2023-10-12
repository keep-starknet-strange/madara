extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{TEST_CONTRACT_ADDRESS, TEST_CONTRACT_CLASS_HASH};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_address = FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).expect("Invalid Contract Address");

    assert_matches!(
        rpc
        .get_class_hash_at(
            BlockId::Number(100),
            test_contract_address,
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

    let unknown_contract_address = FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract Address");

    assert_matches!(
        rpc
        .get_class_hash_at(
            BlockId::Number(0),
            unknown_contract_address,
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_retrieving_class_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_address = FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).expect("Invalid Contract Address");

    assert_eq!(
        rpc.get_class_hash_at(BlockId::Number(0), test_contract_address,).await?,
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).unwrap()
    );

    Ok(())
}
