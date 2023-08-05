use starknet_accounts::{Account, Call, Execution, SingleOwnerAccount};
use starknet_core::types::FieldElement;
use starknet_core::utils::get_selector_from_name;
use starknet_providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_signers::LocalWallet;

use crate::constants::FEE_TOKEN_ADDRESS;

pub fn transfer_tokens<'a>(
    account: &'a SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    recipient: FieldElement,
    transfer_amount: FieldElement,
    nonce: Option<u64>,
) -> Execution<'a, SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    let calls = vec![Call {
        to: fee_token_address,
        selector: get_selector_from_name("transfer").unwrap(),
        calldata: vec![recipient, transfer_amount, FieldElement::ZERO],
    }];

    // TODO: add support for nonce with raw execution e.g https://github.com/0xSpaceShard/starknet-devnet-rs/blob/main/crates/starknet/src/starknet/add_invoke_transaction.rs#L10
    match nonce {
        Some(_nonce) => account.execute(calls),
        None => account.execute(calls),
    }
}
