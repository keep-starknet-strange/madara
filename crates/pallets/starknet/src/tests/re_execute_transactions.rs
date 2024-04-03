use std::sync::Arc;

use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::ExecutableTransaction;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{
    calculate_contract_address, CompiledClassHash, ContractAddress, EntryPointSelector, Nonce, PatriciaKey,
};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt, Fee, TransactionSignature, TransactionVersion};

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::utils::get_contract_class;
use crate::tests::{constants, get_declare_dummy, get_invoke_dummy, set_infinite_tokens};
use crate::Config;

#[test]
fn re_execute_tx_ok() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let invoke_sender_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap()));
        let chain_id = Starknet::chain_id();

        // Deploy

        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = StarkFelt::try_from("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let deploy_tx = {
            use starknet_api::transaction::{DeployAccountTransaction, DeployAccountTransactionV1};

            let tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![]),
                contract_address_salt: ContractAddressSalt(salt),
                constructor_calldata: calldata,
                class_hash: account_class_hash,
            };

            let tx_hash = tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt,
                tx.class_hash,
                &tx.constructor_calldata,
                Default::default(),
            )
            .unwrap();
            blockifier::transaction::transactions::DeployAccountTransaction::new(
                DeployAccountTransaction::V1(tx),
                tx_hash,
                contract_address,
            )
        };

        set_infinite_tokens::<MockRuntime>(&deploy_tx.contract_address);

        // Declare
        let erc20_class_hash = CompiledClassHash(
            StarkFelt::try_from("0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4").unwrap(),
        );
        let erc20_class = get_contract_class("ERC20.json", 0);

        let contract_address = ContractAddress(PatriciaKey(
            StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        ));
        let from_address = StarkFelt::try_from("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let declare_tx = {
            let declare_tx =
                get_declare_dummy(chain_id, Nonce(StarkFelt::ZERO), AccountType::V0(AccountTypeV0Inner::Openzeppelin));
            let tx_hash = declare_tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
            blockifier::transaction::transactions::DeclareTransaction::new(
                declare_tx,
                tx_hash,
                ClassInfo { contract_class: erc20_class, sierra_program_length: usize::MAX, abi_length: usize::MAX },
            )
            .unwrap()
        };

        // Handle l1 message
        let handle_l1_tx = {
            let tx= starknet_api::transaction::L1HandlerTransaction {
            nonce: Nonce(StarkFelt::ONE),
            contract_address,
            entry_point_selector: EntryPointSelector(StarkFelt::try_from(
                "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269", // test_l1_handler_store_under_caller_address
            )
            .unwrap()),
            calldata: Calldata(Arc::new(vec![
                from_address,
                StarkFelt::ONE, // value
            ])),
            version: TransactionVersion(StarkFelt::ZERO),
        };
            let tx_hash = tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
            blockifier::transaction::transactions::L1HandlerTransaction { tx, tx_hash, paid_fee_on_l1: Fee(10) }
        };

        let txs = vec![
            Transaction::AccountTransaction(AccountTransaction::Invoke(
                get_invoke_dummy(Nonce(StarkFelt::ZERO)).into(),
            )),
            Transaction::AccountTransaction(AccountTransaction::Invoke(get_invoke_dummy(Nonce(StarkFelt::ONE)).into())),
            Transaction::AccountTransaction(AccountTransaction::Declare(declare_tx)),
            Transaction::AccountTransaction(AccountTransaction::DeployAccount(deploy_tx)),
            Transaction::L1HandlerTransaction(handle_l1_tx),
        ];

        // Call the function we want to test
        let res = Starknet::re_execute_transactions(txs.clone()).unwrap().unwrap();

        // Storage changes have been reverted
        assert_eq!(Starknet::nonce(invoke_sender_address), Nonce(Felt252Wrapper::ZERO.into()));
        assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash.0), None);
        // All txs are there
        assert_eq!(res.len(), 5);

        // Now let's check the TransactionInfos returned
        let first_invoke_tx_info = match txs.get(0).unwrap() {
            Transaction::AccountTransaction(AccountTransaction::Invoke(invoke_tx)) => {
                let mut state = Starknet::init_cached_state();
                let tx_info = AccountTransaction::Invoke(*invoke_tx)
                    .execute(&mut state, &Starknet::get_block_context(), true, true)
                    .unwrap();
                (tx_info, state.to_state_diff())
            }
            _ => unreachable!(),
        };
        assert_eq!(res[0], first_invoke_tx_info);
        let second_invoke_tx_info = match txs.get(1).unwrap() {
            Transaction::AccountTransaction(AccountTransaction::Invoke(invoke_tx)) => {
                let mut state = Starknet::init_cached_state();
                let tx_info = AccountTransaction::Invoke(*invoke_tx)
                    .execute(&mut state, &Starknet::get_block_context(), true, true)
                    .unwrap();
                (tx_info, state.to_state_diff())
            }
            _ => unreachable!(),
        };
        assert_eq!(res[1], second_invoke_tx_info);
        let declare_tx_info = match txs.get(2).unwrap() {
            Transaction::AccountTransaction(AccountTransaction::Declare(declare_tx)) => {
                let mut state = Starknet::init_cached_state();
                let tx_info = AccountTransaction::Declare(*declare_tx)
                    .execute(&mut state, &Starknet::get_block_context(), true, true)
                    .unwrap();
                (tx_info, state.to_state_diff())
            }
            _ => unreachable!(),
        };
        assert_eq!(res[2], declare_tx_info);
        let deploy_account_tx_info = match txs.get(3).unwrap() {
            Transaction::AccountTransaction(AccountTransaction::DeployAccount(deploy_account_tx)) => {
                let mut state = Starknet::init_cached_state();
                let tx_info = AccountTransaction::DeployAccount(*deploy_account_tx)
                    .execute(&mut state, &Starknet::get_block_context(), true, true)
                    .unwrap();
                (tx_info, state.to_state_diff())
            }
            _ => unreachable!(),
        };
        assert_eq!(res[3], deploy_account_tx_info);
        let handle_l1_message_tx_info = match txs.get(4).unwrap() {
            Transaction::L1HandlerTransaction(l1_tx) => {
                let mut state = Starknet::init_cached_state();
                let tx_info = l1_tx.execute(&mut state, &Starknet::get_block_context(), true, true).unwrap();
                (tx_info, state.to_state_diff())
            }
            _ => unreachable!(),
        };
        assert_eq!(res[4], handle_l1_message_tx_info);
    });
}
