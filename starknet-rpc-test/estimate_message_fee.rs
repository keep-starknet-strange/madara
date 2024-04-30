use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, EthAddress, MsgFromL1, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::constants::{L1_CONTRACT_ADDRESS, TEST_CONTRACT_ADDRESS};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::is_good_error_code;

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
        Err(StarknetProviderError(StarknetError::BlockNotFound))
    );

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

    let estimate_message_fee_err = rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await.unwrap_err();
    assert!(is_good_error_code(&estimate_message_fee_err, 40));

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

    let _fee = rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await?;

    // TODO: uncomment when estimate fee is correctly implemented
    // assert_eq!(fee.gas_consumed, FieldElement::from(17091u128));
    // assert_eq!(fee.gas_price, FieldElement::from(10u128));
    // assert_eq!(fee.overall_fee, FieldElement::ZERO);

    Ok(())
}
