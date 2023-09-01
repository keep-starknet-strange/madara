use starknet_accounts::{Account, Call, Execution, SingleOwnerAccount};
use starknet_core::chain_id;
use starknet_core::types::FieldElement;
use starknet_core::utils::get_selector_from_name;
use starknet_providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_signers::{LocalWallet, SigningKey};

use crate::constants::FEE_TOKEN_ADDRESS;
use crate::RpcAccount;

pub fn create_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(private_key).unwrap()));
    let argent_account_address = FieldElement::from_hex_be(account_address).expect("Invalid Contract Address");
    SingleOwnerAccount::new(rpc, signer, argent_account_address, chain_id::TESTNET)
}

pub trait AccountActions {
    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> Execution<'_, SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>>;
}

impl AccountActions for SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet> {
    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> Execution<'_, SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>> {
        let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

        let calls = vec![Call {
            to: fee_token_address,
            selector: get_selector_from_name("transfer").unwrap(),
            calldata: vec![recipient, transfer_amount, FieldElement::ZERO],
        }];

        // TODO: add support for nonce with raw execution e.g https://github.com/0xSpaceShard/starknet-devnet-rs/blob/main/crates/starknet/src/starknet/add_invoke_transaction.rs#L10
        match nonce {
            Some(_nonce) => self.execute(calls),
            None => self.execute(calls),
        }
    }
}
