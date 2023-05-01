use core::str::FromStr;

use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{EventWrapper, Transaction, TxType};
use sp_core::{H256, U256};

use super::mock::*;
use crate::message::Message;
use crate::{Error, Event};

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let transaction =
            Transaction { version: 1_u8, sender_address: contract_address_bytes, ..Transaction::default() };

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<Test>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_invalid_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_invalid_version.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<Test>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        let tx = Message {
            topics: vec![
                "0xdb80dd488acf86d17c747445b0eabb5d57c541d3bd7b6b87af987858e5066b2b".to_owned(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
                "0x01310e2c127c3b511c5ac0fd7949d544bb4d75b8bc83aaeb357e712ecf582771".to_owned(),
            ],
            data: "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
        }
        .try_into_transaction()
        .unwrap();

        assert_ok!(Starknet::invoke(none_origin.clone(), transaction.clone()));
        assert_ok!(Starknet::consume_l1_message(none_origin, tx));

        let pending = Starknet::pending();
        pretty_assertions::assert_eq!(pending.len(), 2);

        let receipt = &pending.get(0).unwrap().1;
        pretty_assertions::assert_eq!(receipt.actual_fee, U256::from(0));
        pretty_assertions::assert_eq!(receipt.events.len(), 0);
        pretty_assertions::assert_eq!(receipt.transaction_hash, transaction.hash);
        pretty_assertions::assert_eq!(receipt.tx_type, TxType::InvokeTx);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_emit_event.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::invoke(none_origin, transaction.clone()));

        let emitted_event = EventWrapper {
            keys: bounded_vec![
                H256::from_str("0x02d4fbe4956fedf49b5892807e00e7e9eea4680becba55f9187684a69e9424fa").unwrap()
            ],
            data: bounded_vec!(
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
            ),
            from_address: H256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7")
                .unwrap()
                .to_fixed_bytes(),
        };

        System::assert_last_event(Event::StarknetEvent(emitted_event.clone()).into());
        let pending = Starknet::pending();
        let events = Starknet::pending_events();
        let transactions: Vec<Transaction> = pending.clone().into_iter().map(|(transaction, _)| transaction).collect();
        let (_transaction_commitment, event_commitment) =
            commitment::calculate_commitments::<PedersenHasher>(&transactions, &events);
        assert_eq!(
            event_commitment,
            H256::from_str("0x01e95b35377e090a7448a6d09f207557f5fcc962f128ad8416d41c387dda3ec3").unwrap()
        );
        assert_eq!(events.len(), 1);

        pretty_assertions::assert_eq!(pending.len(), 1);

        let receipt = &pending.get(0).unwrap().1;
        pretty_assertions::assert_eq!(receipt.actual_fee, U256::from(0));
        pretty_assertions::assert_eq!(receipt.events.len(), 1);
        pretty_assertions::assert_eq!(receipt.events.get(0).unwrap(), &emitted_event);
        pretty_assertions::assert_eq!(receipt.transaction_hash, transaction.hash);
        pretty_assertions::assert_eq!(receipt.tx_type, TxType::InvokeTx);
    });
}

#[test]
fn given_hardcoded_contract_run_storage_read_and_write_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/storage_read_write.json");
        let transaction =
            transaction_from_json(json_content, include_bytes!("../../../../../resources/account/account.json"))
                .expect("Failed to create Transaction from JSON");

        let target_contract_address =
            U256::from_str("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let storage_var_selector = U256::from(25);

        let mut contract_address_bytes = [0_u8; 32];
        target_contract_address.to_big_endian(&mut contract_address_bytes);
        let mut storage_var_selector_bytes = [0_u8; 32];
        storage_var_selector.to_big_endian(&mut storage_var_selector_bytes);

        assert_ok!(Starknet::invoke(none_origin, transaction));
        assert_eq!(
            Starknet::storage((contract_address_bytes, H256::from_slice(&storage_var_selector_bytes))),
            U256::one()
        );
    });
}
