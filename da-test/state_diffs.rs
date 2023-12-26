extern crate da_test;

use std::vec;

use da_test::fixtures::da_client;
use ethers::types::I256;
use mc_data_availability::DaClient;
use rstest::rstest;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn publish_to_da_layer(
    madara: &ThreadSafeMadaraClient,
    da_client: Box<dyn DaClient + Send + Sync>,
) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let (txs, block_number) = {
        let mut madara_write_lock = madara.write().await;
        // using incorrect private key to generate the wrong signature
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

        let txs = madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                FieldElement::from_hex_be("0x123").unwrap(),
                FieldElement::ONE,
                None,
            ))])
            .await?;
        let block_number = rpc.block_number().await?;

        (txs, block_number)
    };

    assert_eq!(txs.len(), 1);

    let _tx = &txs[0];

    // Check the state diff that has been published to the DA layer
    let published_block_number = da_client.last_published_state().await?;

    assert_eq!(published_block_number, I256::from(block_number));

    Ok(())
}
