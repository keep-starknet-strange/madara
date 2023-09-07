extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, AccountActions};
use starknet_rpc_test::MadaraClient;

#[rstest]
#[tokio::test]
async fn works_with_one_pending_transaction(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    account.transfer_tokens(FieldElement::from_hex_be("0x123").unwrap(), FieldElement::ONE, None).send().await?;

    let pending_txs = rpc.pending_transactions().await?;

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 1);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_500_pending_transactions(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    // loop from 1 to 500
    for nonce in 1..501 {
        let transfer_result = account
            .transfer_tokens(FieldElement::from_hex_be("0x123").unwrap(), FieldElement::ONE, Some(nonce))
            .send()
            .await;
        assert!(transfer_result.is_ok());
    }

    let pending_txs = rpc.pending_transactions().await?;

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 500);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_without_pending_transactions(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let pending_txs = rpc.pending_transactions().await?;

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_eq!(pending_txs.len(), 0);

    Ok(())
}
