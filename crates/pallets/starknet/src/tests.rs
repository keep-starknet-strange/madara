use core::str::FromStr;

use blockifier::test_utils::ACCOUNT_CONTRACT_PATH;
use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::EventWrapper;
use sp_core::{H256, U256};

use crate::mock::*;
use crate::types::Message;
use crate::{Error, Event};

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
    new_test_ext().execute_with(|| {
        let deployer_account = 1;
        let deployer_origin = RuntimeOrigin::signed(deployer_account);
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(Starknet::ping(deployer_origin));
    });
}

#[test]
fn given_normal_conditions_when_current_block_then_returns_correct_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let current_block = Starknet::current_block();

        let expected_current_block = StarknetHeader {
            block_timestamp: 12_000,
            block_number: U256::from(2),
            parent_block_hash: H256::from_str("0x1c2b97b7b9ea91c2cde45bfb115058628c2e1c7aa3fecb51a0cdaf256dc8a310")
                .unwrap(),
            transaction_count: 1,
            // This expected value has been computed in the sequencer test (commitment on a tx hash 0 without
            // signature).
            transaction_commitment: H256::from_str(
                "0x039050b107da7374213fffb38becd5f2d76e51ffa0734bf5c7f8f0477a6f2c22",
            )
            .unwrap(),
            event_count: 2,
            event_commitment: H256::from_str("0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02")
                .unwrap(),
            ..StarknetHeader::default()
        };

        pretty_assertions::assert_eq!(*current_block.header(), expected_current_block)
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/invoke_from_undeclared_sender.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_err!(Starknet::add_invoke_transaction(none_origin, transaction), Error::<Test>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_invalid_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/invoke_invalid_version.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_err!(
            Starknet::add_invoke_transaction(none_origin, transaction),
            Error::<Test>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/invoke.json");
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
        assert_ok!(Starknet::add_invoke_transaction(none_origin.clone(), transaction));

        assert_ok!(Starknet::consume_l1_message(none_origin, tx));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/invoke_emit_event.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::add_invoke_transaction(none_origin.clone(), transaction));

        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x02d4fbe4956fedf49b5892807e00e7e9eea4680becba55f9187684a69e9424fa").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
                ),
                from_address: H256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7")
                    .unwrap()
                    .to_fixed_bytes(),
            })
            .into(),
        );
        let pending = Starknet::pending();

        let (_transaction_commitment, (event_commitment, event_count)) =
            commitment::calculate_commitments::<PedersenHasher>(&pending);
        assert_eq!(
            event_commitment,
            H256::from_str("0x01e95b35377e090a7448a6d09f207557f5fcc962f128ad8416d41c387dda3ec3").unwrap()
        );
        assert_eq!(event_count, 1);
    });
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // TODO: Compute address from salt/hash/calldata/deployer
        let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        let json_content: &str = include_str!("../../../../ressources/transactions/deploy_account.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::add_deploy_account_transaction(none_origin, transaction));

        // Check that the account was created
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address_bytes), class_hash_bytes);
    });
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_twice_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // TODO: Compute address from salt/hash/calldata/deployer
        let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let json_content: &str = include_str!("../../../../ressources/transactions/deploy_account.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::add_deploy_account_transaction(none_origin.clone(), transaction.clone()));

        // Check that the account was created
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address_bytes), class_hash_bytes);

        assert_err!(
            Starknet::add_deploy_account_transaction(none_origin, transaction),
            Error::<Test>::AccountAlreadyDeployed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_undeclared_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let json_content: &str = include_str!("../../../../ressources/transactions/deploy_undeclared_account.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        assert_err!(
            Starknet::add_deploy_account_transaction(none_origin, transaction),
            Error::<Test>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_declare_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/declare_account.json");
        let transaction =
            transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::add_declare_transaction(none_origin, transaction));

        // Check that the class hash was declared
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(class_hash_bytes), account_class);
    });
}

#[test]
fn given_hardcoded_contract_run_declare_twice_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/declare_account.json");
        let transaction =
            transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction from JSON");

        assert_ok!(Starknet::add_declare_transaction(none_origin.clone(), transaction.clone()));

        // Check that the class hash was declared
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(class_hash_bytes), account_class);

        // Second declare should fail
        assert_err!(
            Starknet::add_declare_transaction(none_origin, transaction),
            Error::<Test>::ClassHashAlreadyDeclared
        );
    });
}

#[test]
fn given_hardcoded_contract_run_declare_none_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/declare_account.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        // Cannot declare a class with None
        assert_err!(
            Starknet::add_declare_transaction(none_origin, transaction),
            Error::<Test>::ContractClassMustBeSpecified
        );
    });
}

#[test]
fn given_hardcoded_contract_run_storage_read_and_write_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/storage_read_write.json");
        let transaction =
            transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction from JSON");
        assert_ok!(Starknet::add_invoke_transaction(none_origin, transaction));

        let target_contract_address =
            U256::from_str("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let storage_var_selector = U256::from(25);

        let mut contract_address_bytes = [0_u8; 32];
        target_contract_address.to_big_endian(&mut contract_address_bytes);
        let mut storage_var_selector_bytes = [0_u8; 32];
        storage_var_selector.to_big_endian(&mut storage_var_selector_bytes);
        assert_eq!(
            Starknet::storage((contract_address_bytes, H256::from_slice(&storage_var_selector_bytes))),
            U256::one()
        );
    });
}
