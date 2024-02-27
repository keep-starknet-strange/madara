extern crate starknet_rpc_test;

use std::io::Read;

use assert_matches::assert_matches;
use flate2::read::GzDecoder;
use rstest::rstest;
use starknet_core::types::contract::legacy::{LegacyContractClass, LegacyProgram};
use starknet_core::types::contract::SierraClass;
use starknet_core::types::{BlockId, ContractClass, FlattenedSierraClass, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, TEST_CONTRACT_CLASS_HASH};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_class_hash =
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Address");

    assert_matches!(
        rpc
        .get_class(
            BlockId::Number(100),
            test_contract_class_hash,
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_class_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let unknown_contract_class_hash =
        FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract classh hash");

    assert_matches!(
        rpc
        .get_class(
            BlockId::Number(0),
            unknown_contract_class_hash,
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ClassHashNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "Waiting for issue #1469 to be solved"]
async fn work_ok_retrieving_class_for_contract_version_0(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_class_hash =
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Class Hash");

    let test_contract_class_bytes = include_bytes!("../cairo-contracts/build/test.json");
    let test_contract_class: LegacyContractClass = serde_json::from_slice(test_contract_class_bytes).unwrap();

    assert_matches!(
        rpc
        .get_class(
            BlockId::Number(0),
            test_contract_class_hash,
        ).await?,
        ContractClass::Legacy(c) => {
            // decompress program
            let mut gz = GzDecoder::new(&c.program[..]);
            let mut decompressed_bytes = Vec::new();
            gz.read_to_end(&mut decompressed_bytes).unwrap();
            let program: LegacyProgram = serde_json::from_slice(decompressed_bytes.as_slice())?;
            assert_eq!(
                program.data.len(),
                test_contract_class.program.data.len(),
            );
        }
    );

    Ok(())
}

#[ignore = "conversion between contract class types is incomplete"]
#[rstest]
#[tokio::test]
async fn work_ok_retrieving_class_for_contract_version_1(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_class_hash =
        FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).expect("Invalid Contract Class Hash");

    let test_contract_class_bytes = include_bytes!("../cairo-contracts/build/cairo_1/NoValidateAccount.sierra.json");
    let test_contract_class: SierraClass = serde_json::from_slice(test_contract_class_bytes).unwrap();
    let flattened_test_contract_class: FlattenedSierraClass = test_contract_class.flatten().unwrap();

    assert_matches!(
        rpc
        .get_class(
            BlockId::Number(0),
            test_contract_class_hash
        ).await?,
        ContractClass::Sierra(c) => {
            assert_eq!(
                c.abi,
                flattened_test_contract_class.abi,
            );
            assert_eq!(
                c.sierra_program,
                flattened_test_contract_class.sierra_program,
            );
        }
    );

    Ok(())
}
