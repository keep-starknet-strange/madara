extern crate starknet_rpc_test;

use starknet_accounts::SingleOwnerAccount;
use starknet_core::chain_id;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE};
use starknet_rpc_test::utils::AccountActions;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};
use starknet_signers::{LocalWallet, SigningKey};

#[tokio::test]
async fn work_ok_at_start_and_with_new_blocks() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
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

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(SIGNER_PRIVATE).unwrap()));
    let argent_account_address = FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).expect("Invalid Contract Address");
    let account = SingleOwnerAccount::new(rpc, signer, argent_account_address, chain_id::TESTNET);

    madara
        .create_block_with_txs(vec![account.transfer_tokens(
            argent_account_address,
            FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
            None,
        )])
        .await?;
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 2);
    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x035d394a3808b432702261c2d66f1cf2f111dd0c691ad2a8246b86a5088f73fa").unwrap()
    );

    Ok(())
}
