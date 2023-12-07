extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{BlockId, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, read_erc20_balance, AccountActions, U256};
use starknet_rpc_test::{SendTransactionError, Transaction};

#[rstest]
#[tokio::test]
async fn publishes_to_da_layer(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let txs = {
        let mut madara_write_lock = madara.write().await;
        // using incorrect private key to generate the wrong signature
        let account = build_single_owner_account(&rpc, "0x1234", ARGENT_CONTRACT_ADDRESS, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                FieldElement::from_hex_be("0x123").unwrap(),
                FieldElement::ONE,
                None,
            ))])
            .await?
    };

    assert_eq!(txs.len(), 1);

    let tx = &txs[0];

    // Check the state diff that has been published to the DA layer

    Ok(())
}
