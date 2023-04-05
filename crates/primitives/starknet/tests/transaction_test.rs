use core::str::FromStr;
use std::collections::HashMap;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CachedState;
use blockifier::transaction::objects::AccountTransactionContext;
use frame_support::bounded_vec;
use hex::FromHex;
use mp_starknet::execution::{CallEntryPointWrapper, EntryPointTypeWrapper};
use mp_starknet::state::DictStateReader;
use mp_starknet::transaction::types::{Transaction, TxType};
use sp_core::{H256, U256};
use starknet_api::api_core::{
    ClassHash, ContractAddress as StarknetContractAddress, ContractAddress, Nonce, PatriciaKey,
};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Fee, InvokeTransactionV1, TransactionHash, TransactionSignature, TransactionVersion};
use starknet_api::{patricia_key, stark_felt};

#[test]

fn test_verify_nonce() {
    let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();
    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let tx = Transaction::new(
        U256::from(1),
        H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap(),
        bounded_vec![
            H256::from_str("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
            H256::from_str("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap()
        ],
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                H256::from_str("0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c").unwrap(), /* Contract address */
                H256::from_str("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* Selector */
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* Length */
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), // Value
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    let _txtype = TxType::InvokeTx;

    // create a state
    let class_hash = "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let contract_address = "0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let class_hash_to_class = HashMap::from([(ClassHash(stark_felt!(class_hash)), ContractClass::default())]);
    let address_to_class_hash =
        HashMap::from([(ContractAddress(patricia_key!(contract_address)), ClassHash(stark_felt!(class_hash)))]);
    let mut state =
        CachedState::new(DictStateReader { class_hash_to_class, address_to_class_hash, ..Default::default() });

    let invoke_tx = InvokeTransactionV1 {
        transaction_hash: TransactionHash(StarkFelt::new(tx.hash.0).unwrap()),
        max_fee: Fee(2),

        signature: TransactionSignature(
            tx.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
        ),
        nonce: Nonce(StarkFelt::new(tx.nonce.into()).unwrap()),
        sender_address: StarknetContractAddress::try_from(StarkFelt::new(tx.sender_address).unwrap()).unwrap(),
        calldata: tx.call_entrypoint.to_starknet_call_entry_point().calldata,
    };
    let account_tx_context = AccountTransactionContext {
        transaction_hash: invoke_tx.transaction_hash,
        max_fee: invoke_tx.max_fee,
        version: TransactionVersion(StarkFelt::from(1)),
        signature: invoke_tx.signature.clone(),
        nonce: invoke_tx.nonce,
        sender_address: invoke_tx.sender_address,
    };

    // Test for a valid nonce
    let result = tx.verify_nonce(&account_tx_context, &mut state);
    assert!(result.is_ok());

    // Test for an invalid nonce
    let account_tx_context_invalid_nonce = AccountTransactionContext { nonce: Nonce(2.into()), ..account_tx_context };
    let result = tx.verify_nonce(&account_tx_context_invalid_nonce, &mut state);
    assert!(result.is_err());
}
