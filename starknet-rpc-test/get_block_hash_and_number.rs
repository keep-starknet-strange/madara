extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction};

#[rstest]
#[tokio::test]
async fn work_ok_at_start_and_with_new_blocks(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad").unwrap()
    );
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 0);

    madara.create_empty_block().await?;
    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x001d68e058e03162e4864ef575445c38deea4fad6b56974ef9012e8429c2e7b9").unwrap()
    );
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 1);

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let token_transfer_tx = Transaction::Execution(account.transfer_tokens(
        FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).expect("Invalid Contract Address"),
        FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
        None,
    ));

    madara.create_block_with_txs(vec![token_transfer_tx]).await?;
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 2);
    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x049b84477d7b0e2f6d6e3cf7dffcb8e5e12b6bb07f673daf7e85b06e69fd041b").unwrap()
    );

    Ok(())
}
