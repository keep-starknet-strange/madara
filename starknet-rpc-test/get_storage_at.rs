extern crate starknet_rpc_test;

use starknet_core::types::{BlockId, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};

#[tokio::test]
async fn work_ok_at_storage() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    // Return an error for a nonexisting block
    let err = rpc
        .get_storage_at(
            FieldElement::from_hex_be("0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7").unwrap(),
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Number(100),
        )
        .await;
    if let Err(error) = err {
        if let StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. }) = error
        {
            assert_eq!(code, StarknetError::BlockNotFound);
        } else {
            panic!("Unexpected error: {:?}", error);
        }
    } else {
        panic!("Expected an error for a nonexisting block, but got Ok");
    }

    // Return an error for a non-existing contract
    let err = rpc
        .get_storage_at(
            FieldElement::from_hex_be("0x051e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00").unwrap(),
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Number(0),
        )
        .await;
    if let Err(error) = err {
        if let StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. }) = error
        {
            assert_eq!(code, StarknetError::ContractNotFound);
        } else {
            panic!("Unexpected error: {:?}", error);
        }
    } else {
        panic!("Expected an error for a non-existing contract, but got Ok");
    }

    // Return the correct value previously stored in a contract
    assert_eq!(
        rpc.get_storage_at(
            FieldElement::from_hex_be("0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7").unwrap(),
            FieldElement::from_hex_be("0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
            BlockId::Number(0)
        )
        .await?,
        FieldElement::from_hex_be("0xffffffffffffffffffffffffffffffff").unwrap()
    );

    Ok(())
}
