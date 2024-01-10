use alloc::sync::Arc;
use core::str::FromStr;
use std::path::PathBuf;
use std::{env, fs};

use blockifier::execution::contract_class::ContractClass;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use mp_transactions::{InvokeTransaction, InvokeTransactionV1};
use starknet_api::api_core::EntryPointSelector;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
use starknet_crypto::{sign, FieldElement};

use super::constants::{ACCOUNT_PRIVATE_KEY, K};
use crate::genesis_loader::read_contract_class_from_json;
use crate::types::BuildTransferInvokeTransaction;

pub fn get_contract_class(resource_path: &str, version: u8) -> ContractClass {
    let cargo_dir = String::from(env!("CARGO_MANIFEST_DIR"));
    let build_path = match version {
        0 => "/../../../cairo-contracts/build/",
        1 => "/../../../cairo-contracts/build/cairo_1/",
        _ => unimplemented!("Unsupported version {} to get contract class", version),
    };
    let full_path = cargo_dir + build_path + resource_path;
    let full_path: PathBuf = [full_path].iter().collect();
    let raw_contract_class = fs::read_to_string(full_path).unwrap();
    read_contract_class_from_json(&raw_contract_class, version)
}

pub fn sign_message_hash_braavos(
    tx_hash: Felt252Wrapper,
    actual_impl_hash: Felt252Wrapper,
    signer_model: &[Felt252Wrapper; 7],
) -> Vec<Felt252Wrapper> {
    // struct SignerModel {
    //     signer_0: felt,
    //     signer_1: felt,
    //     signer_2: felt,
    //     signer_3: felt,
    //     type: felt,
    //     reserved_0: felt,
    //     reserved_1: felt,
    // }
    let mut elements = vec![tx_hash.0, actual_impl_hash.0];
    elements.extend_from_slice(&signer_model.iter().map(|e| e.0).collect::<Vec<FieldElement>>());
    let braavos_hash = PedersenHasher::compute_hash_on_elements(&elements);

    let mut signatures = sign_message_hash(Felt252Wrapper(braavos_hash));
    signatures.push(actual_impl_hash);
    signatures.extend_from_slice(signer_model);
    signatures
}

pub fn sign_message_hash(hash: Felt252Wrapper) -> Vec<Felt252Wrapper> {
    let signature = sign(
        &FieldElement::from_str(ACCOUNT_PRIVATE_KEY).unwrap(),
        &FieldElement::from(hash),
        &FieldElement::from_str(K).unwrap(),
    )
    .unwrap();
    vec![signature.r.into(), signature.s.into()]
}

pub fn build_transfer_invoke_transaction(request: BuildTransferInvokeTransaction) -> InvokeTransaction {
    InvokeTransactionV1 {
        max_fee: u128::MAX,
        signature: vec![],
        nonce: request.nonce,
        sender_address: request.sender_address,
        calldata: vec![
            request.token_address, // Token address
            Felt252Wrapper::from_hex_be(
                "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            )
            .unwrap(), /* transfer
                                    * selector */
            Felt252Wrapper::THREE, // Calldata len
            request.recipient,     // recipient
            request.amount_low,    // initial supply low
            request.amount_high,   // initial supply high
        ],
        offset_version: false,
    }
    .into()
}

pub fn build_get_balance_contract_call(account_address: StarkFelt) -> (EntryPointSelector, Calldata) {
    let balance_of_selector = EntryPointSelector(
        StarkFelt::try_from("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap(),
    );
    let calldata = Calldata(Arc::new(vec![
        account_address, // owner address
    ]));

    (balance_of_selector, calldata)
}
