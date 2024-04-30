use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};

use blockifier::execution::contract_class::{ClassInfo, ContractClass};
use blockifier::transaction::transactions::DeclareTransaction as BlockifierDeclareTransaction;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata, DeclareTransaction as StarknetApiDeclareTransaction, DeclareTransactionV0V1, Fee, TransactionHash,
    TransactionSignature,
};
use starknet_crypto::{sign, FieldElement};

use super::constants::{ACCOUNT_PRIVATE_KEY, K};
use super::mock::{get_account_address, AccountType};
use crate::genesis_loader::read_contract_class_from_json;

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
    elements.extend_from_slice(
        &signer_model.iter().map(|e| Felt252Wrapper::from(*e).into()).collect::<Vec<FieldElement>>(),
    );
    let braavos_hash = PedersenHasher::compute_hash_on_elements(&elements);

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

pub(crate) fn create_declare_erc20_v1_transaction(
    chain_id: Felt252Wrapper,
    account_type: AccountType,
    sender_address: Option<ContractAddress>,
    signature: Option<TransactionSignature>,
    nonce: Option<Nonce>,
) -> BlockifierDeclareTransaction {
    let sender_address = sender_address.unwrap_or_else(|| get_account_address(None, account_type));

    let erc20_class = get_contract_class("ERC20.json", 0);
    let erc20_class_hash =
        ClassHash(StarkFelt::try_from("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap());

    let mut tx = StarknetApiDeclareTransaction::V1(DeclareTransactionV0V1 {
        max_fee: Fee(u128::MAX),
        signature: Default::default(),
        nonce: nonce.unwrap_or_default(),
        class_hash: erc20_class_hash,
        sender_address,
    });

    let tx_hash = tx.compute_hash(chain_id, false);
    // Force to do that because ComputeTransactionHash cannot be implemented on DeclareTransactionV0V1
    // directly...
    if let StarknetApiDeclareTransaction::V1(tx) = &mut tx {
        tx.signature = signature.unwrap_or_else(|| sign_message_hash(tx_hash));
    }

    BlockifierDeclareTransaction::new(tx, tx_hash, ClassInfo::new(&erc20_class, 0, 1).unwrap()).unwrap()
}

pub(crate) fn create_declare_erc721_v1_transaction(
    chain_id: Felt252Wrapper,
    account_type: AccountType,
    sender_address: Option<ContractAddress>,
    signature: Option<TransactionSignature>,
    nonce: Option<Nonce>,
) -> BlockifierDeclareTransaction {
    let sender_address = sender_address.unwrap_or_else(|| get_account_address(None, account_type));

    let erc721_class = get_contract_class("ERC721.json", 0);
    let erc721_class_hash =
        ClassHash(StarkFelt::try_from("0x077cc28ed3c661419fda16bf120fb81f1f8f28617f5543b05a86d63b0926bbf4").unwrap());

    let mut tx = StarknetApiDeclareTransaction::V1(DeclareTransactionV0V1 {
        max_fee: Fee(u128::MAX),
        signature: Default::default(),
        nonce: nonce.unwrap_or_default(),
        class_hash: erc721_class_hash,
        sender_address,
    });

    let tx_hash = tx.compute_hash(chain_id, false);
    // Force to do that because ComputeTransactionHash cannot be implemented on DeclareTransactionV0V1
    // directly...
    if let StarknetApiDeclareTransaction::V1(tx) = &mut tx {
        tx.signature = signature.unwrap_or_else(|| sign_message_hash(tx_hash));
    }

    BlockifierDeclareTransaction::new(tx, tx_hash, ClassInfo::new(&erc721_class, 0, 1).unwrap()).unwrap()
}
