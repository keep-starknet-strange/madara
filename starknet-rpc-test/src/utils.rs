use std::future::Future;
use std::sync::Arc;

use starknet_accounts::{Account, AccountFactory, Call, OpenZeppelinAccountFactory, SingleOwnerAccount};
use starknet_core::chain_id;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::{BlockId, BlockTag, EmittedEvent, Event, FieldElement, FunctionCall, MsgToL1};
use starknet_core::utils::get_selector_from_name;
use starknet_providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_providers::Provider;
use starknet_signers::{LocalWallet, SigningKey};

use crate::constants::{FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE};
use crate::{
    RpcAccount, RpcOzAccountFactory, TransactionAccountDeployment, TransactionDeclaration, TransactionExecution,
    TransactionLegacyDeclaration,
};

pub struct U256 {
    pub high: FieldElement,
    pub low: FieldElement,
}

pub fn build_single_owner_account<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    account_address: &str,
    is_legacy: bool,
) -> RpcAccount<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(private_key).unwrap()));
    let account_address = FieldElement::from_hex_be(account_address).expect("Invalid Contract Address");
    let execution_encoding = if is_legacy {
        starknet_accounts::ExecutionEncoding::Legacy
    } else {
        starknet_accounts::ExecutionEncoding::New
    };
    SingleOwnerAccount::new(rpc, signer, account_address, chain_id::TESTNET, execution_encoding)
}

pub async fn read_erc20_balance(
    rpc: &JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    account_address: FieldElement,
) -> Vec<FieldElement> {
    rpc.call(
        FunctionCall {
            contract_address,
            entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
            calldata: vec![account_address],
        },
        BlockId::Tag(BlockTag::Latest),
    )
    .await
    .unwrap()
}

pub async fn build_oz_account_factory<'a>(
    rpc: &'a JsonRpcClient<HttpTransport>,
    private_key: &str,
    class_hash: FieldElement,
) -> RpcOzAccountFactory<'a> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(private_key).unwrap()));
    OpenZeppelinAccountFactory::new(class_hash, chain_id::TESTNET, signer, rpc).await.unwrap()
}

pub fn build_deploy_account_tx<'a>(
    oz_factory: &'a RpcOzAccountFactory,
    contract_address_salt: FieldElement,
) -> TransactionAccountDeployment<'a> {
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();
    oz_factory.deploy(contract_address_salt).max_fee(max_fee)
}

pub trait AccountActions {
    fn transfer_tokens_u256(
        &self,
        recipient: FieldElement,
        transfer_amount: U256,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn declare_contract(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (TransactionDeclaration, FieldElement, FieldElement);

    fn declare_legacy_contract(&self, path_to_compiled_contract: &str) -> (TransactionLegacyDeclaration, FieldElement);
}

impl AccountActions for SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet> {
    fn transfer_tokens_u256(
        &self,
        recipient: FieldElement,
        transfer_amount: U256,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

        let calls = vec![Call {
            to: fee_token_address,
            selector: get_selector_from_name("transfer").unwrap(),
            calldata: vec![recipient, transfer_amount.low, transfer_amount.high],
        }];

        // starknet-rs calls estimateFee with incorrect version which throws an error
        let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

        // TODO: add support for nonce with raw execution e.g https://github.com/0xSpaceShard/starknet-devnet-rs/blob/main/crates/starknet/src/starknet/add_invoke_transaction.rs#L10
        match nonce {
            Some(nonce) => self.execute(calls).max_fee(max_fee).nonce(nonce.into()),
            None => self.execute(calls).max_fee(max_fee),
        }
    }

    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        self.transfer_tokens_u256(recipient, U256 { high: FieldElement::ZERO, low: transfer_amount }, nonce)
    }

    fn declare_contract(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (TransactionDeclaration, FieldElement, FieldElement) {
        let sierra: SierraClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_sierra).unwrap(),
        )
        .unwrap();
        let casm: CompiledClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_casm).unwrap(),
        )
        .unwrap();
        let compiled_class_hash = casm.class_hash().unwrap();
        (
            self.declare(sierra.clone().flatten().unwrap().into(), compiled_class_hash)
				// starknet-rs calls estimateFee with incorrect version which throws an error
                .max_fee(FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap()),
            sierra.class_hash().unwrap(),
            compiled_class_hash,
        )
    }

    fn declare_legacy_contract(&self, path_to_compiled_contract: &str) -> (TransactionLegacyDeclaration, FieldElement) {
        let contract_artifact: LegacyContractClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_compiled_contract).unwrap(),
        )
        .unwrap();
        (
            self.declare_legacy(Arc::new(contract_artifact.clone()))
			 // starknet-rs calls estimateFee with incorrect version which throws an error
			 .max_fee(FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap()),
            contract_artifact.class_hash().unwrap(),
        )
    }
}

pub fn assert_eq_msg_to_l1(l1: Vec<MsgToL1>, l2: Vec<MsgToL1>) {
    assert_eq!(l1.len(), l2.len());
    for (m1, m2) in l1.iter().zip(l2.iter()) {
        assert_eq!(m1.from_address, m2.from_address);
        assert_eq!(m1.payload, m2.payload);
        assert_eq!(m1.to_address, m2.to_address);
    }
}

pub fn assert_eq_event(l1: Vec<Event>, l2: Vec<Event>) {
    assert_eq!(l1.len(), l2.len());
    for (e1, e2) in l1.iter().zip(l2.iter()) {
        assert_eq!(e1.data, e2.data);
        assert_eq!(e1.from_address, e2.from_address);
        assert_eq!(e1.keys, e2.keys);
    }
}

pub fn assert_eq_emitted_event(l1: &[EmittedEvent], l2: &[EmittedEvent]) -> bool {
    assert_eq!(l1.len(), l2.len());
    l1.iter().zip(l2.iter()).all(|(e1, e2)| {
        e1.data == e2.data
            || e1.from_address == e2.from_address
            || e1.keys == e2.keys
            || e1.block_hash == e2.block_hash
            || e1.block_number == e2.block_number
            || e1.transaction_hash == e2.transaction_hash
    })
}

pub async fn assert_poll<F, Fut>(f: F, polling_time_ms: u64, max_poll_count: u32)
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    for _poll_count in 0..max_poll_count {
        if f().await {
            return; // The provided function returned true, exit safely.
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(polling_time_ms)).await;
    }

    panic!("Max poll count exceeded.");
}
