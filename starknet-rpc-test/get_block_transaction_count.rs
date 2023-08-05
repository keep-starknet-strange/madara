extern crate starknet_rpc_test;

use starknet_accounts::SingleOwnerAccount;
use starknet_core::chain_id;
use starknet_core::types::{BlockId, BlockTag};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE};
use starknet_rpc_test::utils::transfer_tokens;
use starknet_rpc_test::{BlockCreation, ExecutionStrategy, MadaraClient};
use starknet_signers::{LocalWallet, SigningKey};

#[tokio::test]
async fn work_ok_with_and_without_txs() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 0);

    madara.create_block(vec![], BlockCreation::default()).await?;
    assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 0);

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(SIGNER_PRIVATE).unwrap()));
    let argent_account_address = FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).expect("Invalid Contract Address");
    let account = SingleOwnerAccount::new(rpc, signer, argent_account_address, chain_id::TESTNET);

    madara
        .create_block(
            vec![transfer_tokens(
                &account,
                argent_account_address,
                FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
                None,
            )],
            BlockCreation::new(None, true),
        )
        .await?;
    assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 1);

    // TODO: Uncomment when raw execution is supported
    // madara
    //     .create_block(
    //         vec![
    //             transfer_tokens(
    //                 &account,
    //                 argent_account_address,
    //                 FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
    //                 Some(1),
    //             ),
    //             transfer_tokens(
    //                 &account,
    //                 argent_account_address,
    //                 FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
    //                 Some(2),
    //             ),
    //         ],
    //         BlockCreation::new(None, true),
    //     )
    //     .await?;

    // assert_eq!(rpc.get_block_transaction_count(BlockId::Tag(BlockTag::Latest)).await?, 2);

    Ok(())
}
