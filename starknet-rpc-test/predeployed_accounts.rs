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
async fn get_predeployed_accounts_list(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    pub struct PredeployedAccount {
        pub contract_address: FieldElement,
        pub contract_class: ContractClass,
        pub balance: FieldElement,
        pub private_key: Option<FieldElement>,
    }

    let rpc = madara.get_starknet_client().await;

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).expect("Invalid Contract Address");

    let predeployed_accounts_list = rpc.predeployed_accounts().await.unwrap();
    assert_matches!(predeployed_accounts_list.len(), 4);

    Ok(())
}
