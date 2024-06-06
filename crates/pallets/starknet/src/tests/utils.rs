use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};

use blockifier::execution::contract_class::ContractClass;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Calldata, Fee, TransactionHash, TransactionSignature};
use starknet_crypto::{sign, FieldElement};

use super::constants::{ACCOUNT_PRIVATE_KEY, K};
use super::mock::default_mock::*;
use crate::genesis_loader::read_contract_class_from_json;
use crate::tests::mock::get_storage_key;
use crate::StorageView;

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
    tx_hash: TransactionHash,
    actual_impl_hash: StarkHash,
    signer_model: &[StarkFelt; 7],
) -> TransactionSignature {
    // struct SignerModel {
    //     signer_0: felt,
    //     signer_1: felt,
    //     signer_2: felt,
    //     signer_3: felt,
    //     type: felt,
    //     reserved_0: felt,
    //     reserved_1: felt,
    // }
    let mut elements: Vec<FieldElement> =
        vec![Felt252Wrapper::from(tx_hash).into(), Felt252Wrapper::from(actual_impl_hash).into()];

    elements.extend(signer_model.iter().map(|e| {
        let wrapper: Felt252Wrapper = Felt252Wrapper::from(*e);
        let field_element: FieldElement = wrapper.into();
        field_element
    }));

    let braavos_hash = PedersenHasher::compute_hash_on_elements(elements);

    let mut signatures = sign_message_hash(Felt252Wrapper(braavos_hash).into());
    signatures.0.push(actual_impl_hash);
    signatures.0.extend_from_slice(signer_model);
    signatures
}

pub fn sign_message_hash(hash: TransactionHash) -> TransactionSignature {
    let signature = sign(
        &FieldElement::from_str(ACCOUNT_PRIVATE_KEY).unwrap(),
        &Felt252Wrapper::from(hash).into(),
        &FieldElement::from_str(K).unwrap(),
    )
    .unwrap();

    TransactionSignature(vec![Felt252Wrapper(signature.r).into(), Felt252Wrapper(signature.s).into()])
}

pub fn build_transfer_invoke_transaction(
    chain_id: Felt252Wrapper,
    request: BuildTransferInvokeTransaction,
) -> blockifier::transaction::transactions::InvokeTransaction {
    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: Fee(u128::MAX),
        signature: TransactionSignature(vec![]),
        nonce: request.nonce,
        sender_address: request.sender_address,
        calldata: Calldata(Arc::new(vec![
            request.token_address.0.0, // Token address
            StarkFelt::try_from("0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e").unwrap(), /* transfer
                                                                                                                 * selector */
            StarkFelt::THREE,      // Calldata len
            request.recipient.0.0, // recipient
            request.amount_low,    // initial supply low
            request.amount_high,   // initial supply high
        ])),
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

/// Build invoke transaction for transfer utils
pub struct BuildTransferInvokeTransaction {
    pub sender_address: ContractAddress,
    pub token_address: ContractAddress,
    pub recipient: ContractAddress,
    pub amount_low: StarkFelt,
    pub amount_high: StarkFelt,
    pub nonce: Nonce,
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

pub fn create_resource_bounds() -> starknet_api::transaction::ResourceBoundsMapping {
    let mut map = BTreeMap::new();
    map.insert(
        starknet_api::transaction::Resource::L1Gas,
        starknet_api::transaction::ResourceBounds { max_amount: 10000, max_price_per_unit: 12000 },
    );
    map.insert(
        starknet_api::transaction::Resource::L2Gas,
        starknet_api::transaction::ResourceBounds { max_amount: 10000, max_price_per_unit: 12000 },
    );
    starknet_api::transaction::ResourceBoundsMapping(map)
}

pub fn get_balance_contract_call(
    sender_address: ContractAddress,
    contract_address: ContractAddress,
) -> Vec<Felt252Wrapper> {
    let call_args = build_get_balance_contract_call(sender_address.0.0);
    Starknet::call_contract(contract_address, call_args.0, call_args.1).unwrap()
}

pub fn set_account_erc20_balance_to_zero(account_address: ContractAddress, erc20_contract_address: ContractAddress) {
    // ContractAddress to FieldElement
    let field_contract_address = FieldElement::from_bytes_be(&account_address.key().0).unwrap();

    // Get balance variable key
    let balance_low_storage_key =
        get_storage_key(&erc20_contract_address, "ERC20_balances", &[field_contract_address], 0);

    let balance_high_storage_key =
        get_storage_key(&erc20_contract_address, "ERC20_balances", &[field_contract_address], 1);

    // Set balance storage value to zero
    StorageView::<MockRuntime>::insert(balance_low_storage_key, StarkFelt::try_from("0").unwrap());
    StorageView::<MockRuntime>::insert(balance_high_storage_key, StarkFelt::try_from("0").unwrap());
}
