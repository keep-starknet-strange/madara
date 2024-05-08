use std::sync::Arc;

use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use mp_felt::Felt252Wrapper;
use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::{
    constants, create_l1_handler_transaction, get_declare_dummy, get_deploy_account_dummy, get_invoke_dummy,
    set_infinite_tokens,
};
use crate::types::CasmClassHash;

#[test]
fn re_execute_tx_ok() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let invoke_sender_address: ContractAddress =
            Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap().into();
        let txs = get_test_txs();

        let txs_to_ignore: Vec<Transaction> = vec![];
        let erc20_class_hash: CasmClassHash =
            Felt252Wrapper::from_hex_be("0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4")
                .unwrap()
                .into();

        // Call the function we want to test
        let res = Starknet::re_execute_transactions(txs_to_ignore, txs.clone(), false).unwrap().unwrap();

        // Storage changes have been reverted
        assert_eq!(Starknet::nonce(invoke_sender_address), Nonce(Felt252Wrapper::ZERO.into()));
        assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), None);
        // All txs are there
        assert_eq!(res.len(), 5);
    });
}

#[test]
fn re_execute_tx_with_a_transfer_ok() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let chain_id = Starknet::chain_id();
        let invoke_sender_address: ContractAddress =
            Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap().into();
        let txs = get_test_txs();
        let erc20_class_hash: CasmClassHash =
            Felt252Wrapper::from_hex_be("0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4")
                .unwrap()
                .into();

        let transfer_tx = Transaction::AccountTransaction(AccountTransaction::Invoke(get_invoke_dummy(
            chain_id,
            Nonce(StarkFelt::TWO),
        )));

        // Call the function we want to test
        let res = Starknet::re_execute_transactions(txs.clone(), vec![transfer_tx.clone()], false).unwrap().unwrap();

        // Storage changes have been reverted
        assert_eq!(Starknet::nonce(invoke_sender_address), Nonce(Felt252Wrapper::ZERO.into()));
        assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), None);
        // Here we only got the transfer tx
        assert_eq!(res.len(), 1);
    });
}

fn get_test_txs() -> Vec<Transaction> {
    let chain_id = Starknet::chain_id();

    // Deploy

    // TEST ACCOUNT CONTRACT
    // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
    let salt = ContractAddressSalt(
        StarkFelt::try_from("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap(),
    );

    let deploy_tx =
        get_deploy_account_dummy(chain_id, Nonce::default(), salt, AccountType::V0(AccountTypeV0Inner::NoValidate));
    let address = deploy_tx.contract_address;
    set_infinite_tokens::<MockRuntime>(&address);

    // Declare

    let declare_tx =
        get_declare_dummy(chain_id, Nonce(StarkFelt::ZERO), AccountType::V0(AccountTypeV0Inner::Openzeppelin));

    let contract_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
    ));
    let from_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap()));

    // Handle l1 message
    let handle_l1_tx = create_l1_handler_transaction(
        chain_id,
        Nonce(StarkFelt::ONE),
        Some(contract_address),
        Some(EntryPointSelector(StarkFelt::try_from(
            "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269", /* test_l1_handler_store_under_caller_address */
        ).unwrap())),
        Some(Calldata(Arc::new(
        vec![
                    from_address.0.0,
                    StarkFelt::ONE, // value
                ]))),
    );

    vec![
        Transaction::AccountTransaction(AccountTransaction::Invoke(get_invoke_dummy(chain_id, Nonce(StarkFelt::ZERO)))),
        Transaction::AccountTransaction(AccountTransaction::Invoke(get_invoke_dummy(chain_id, Nonce(StarkFelt::ONE)))),
        Transaction::AccountTransaction(AccountTransaction::Declare(declare_tx)),
        Transaction::AccountTransaction(AccountTransaction::DeployAccount(deploy_tx)),
        Transaction::L1HandlerTransaction(handle_l1_tx),
    ]
}
