extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, EthAddress, MsgFromL1, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{L1_CONTRACT_ADDRESS, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: get_selector_from_name("sqrt").unwrap(),
        payload: vec![FieldElement::from_hex_be("0x0").unwrap()],
    };

    assert_matches!(
        rpc.estimate_message_fee(message, BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code:
MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound     );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_message_fail(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: get_selector_from_name("sqrt").unwrap(),
        payload: vec![FieldElement::from_hex_be("0x0").unwrap()],
    };

    assert_matches!(
        rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code:
MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError     );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: FieldElement::from_hex_be(
            "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269",
        )
        .unwrap(),
        payload: vec![1u64.into()],
    };

    let fee = rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await?;

    assert_eq!(fee.gas_consumed, 17091);
    assert_eq!(fee.gas_price, 10);
    assert_eq!(fee.overall_fee, 0);

    Ok(())
}
