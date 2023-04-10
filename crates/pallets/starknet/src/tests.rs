use core::str::FromStr;

use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH, ERC20_CONTRACT_PATH};
use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use hex::FromHex;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::execution::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{EventWrapper, Transaction};
use sp_core::{H256, U256};
use sp_runtime::DispatchError;

use crate::mock::*;
use crate::types::Message;
use crate::{Error, Event};

#[test]
fn should_calculate_contract_addr_correct() {
    let (addr, _, _) = account_helper(TEST_ACCOUNT_SALT);
    let exp = <[u8; 32]>::from_hex("00b72536305f9a17ed8c0d9abe80e117164589331c3e9547942a830a99d3a5e9").unwrap();
    assert_eq!(addr, exp);
}

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let (addr, _, _) = account_helper("0x00000000000000000000000000000000000000000000000000000000DEADBEEF");
    let exp = <[u8; 32]>::from_hex("00b72536305f9a17ed8c0d9abe80e117164589331c3e9547942a830a99d3a5e9").unwrap();
    assert_ne!(addr, exp);
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

        let json_content: &str = include_str!("../../../../ressources/transactions/invoke_invalid_version.json");
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

        assert_ok!(Starknet::invoke(none_origin.clone(), transaction));
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

        assert_ok!(Starknet::invoke(none_origin, transaction));

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
fn given_hardcoded_contract_run_storage_read_and_write_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../ressources/transactions/storage_read_write.json");
        let transaction =
            transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction from JSON");

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

#[test]
fn given_contract_run_deploy_account_tx_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt);

        let transaction = Transaction {
            sender_address: test_addr,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(account_class_hash),
                EntryPointTypeWrapper::External,
                None,
                BoundedVec::try_from(calldata.clone().into_iter().map(|x| U256::from(x)).collect::<Vec<U256>>())
                    .unwrap(),
                test_addr,
                test_addr,
            ),
            contract_address_salt: Some(H256::from_str(salt).unwrap()),
            ..Transaction::default()
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_tx_twice_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt);

        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let transaction = Transaction {
            sender_address: test_addr,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(account_class_hash),
                EntryPointTypeWrapper::External,
                None,
                BoundedVec::try_from(calldata.clone().into_iter().map(|x| U256::from(x)).collect::<Vec<U256>>())
                    .unwrap(),
                test_addr,
                test_addr,
            ),
            contract_address_salt: Some(H256::from_str(salt).unwrap()),
            ..Transaction::default()
        };

        assert_ok!(Starknet::deploy_account(none_origin.clone(), transaction.clone()));
        // Check that the account was created
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr), account_class_hash);
        assert_err!(Starknet::deploy_account(none_origin, transaction), Error::<Test>::AccountAlreadyDeployed);
    });
}

#[test]
fn given_contract_run_deploy_account_tx_undeclared_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let rand_address =
            <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000001234").unwrap();
        let undeclared_class_hash =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000BEEFDEAD").unwrap();

        let transaction = Transaction {
            sender_address: rand_address,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(undeclared_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                rand_address,
                rand_address,
            ),
            ..Transaction::default()
        };

        assert_err!(Starknet::deploy_account(none_origin, transaction), Error::<Test>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let erc20_class = ContractClassWrapper::from(get_contract_class(ERC20_CONTRACT_PATH));
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let mut transaction = Transaction {
            sender_address: account_addr,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                account_addr,
                account_addr,
            ),
            ..Transaction::default()
        };
        // Cannot declare a class with None
        assert_err!(
            Starknet::declare(none_origin.clone(), transaction.clone()),
            Error::<Test>::ContractClassMustBeSpecified
        );

        transaction.contract_class = Some(erc20_class.clone());

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn given_root_when_set_fee_token_address_then_fee_token_address_is_updated() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let root_origin = RuntimeOrigin::root();
        let current_fee_token_address = Starknet::fee_token_address();
        let new_fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();

        assert_ok!(Starknet::set_fee_token_address(root_origin.clone(), new_fee_token_address));
        System::assert_last_event(
            Event::FeeTokenAddressChanged { old_fee_token_address: current_fee_token_address, new_fee_token_address }
                .into(),
        );
    })
}

#[test]
fn given_non_root_when_set_fee_token_address_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let non_root_origin = RuntimeOrigin::signed(1);
        let new_fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();
        assert_err!(Starknet::set_fee_token_address(non_root_origin, new_fee_token_address), DispatchError::BadOrigin);
    })
}
