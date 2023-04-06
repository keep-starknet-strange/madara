use core::str::FromStr;

use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH};
use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use hex::FromHex;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::execution::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::{EventWrapper, Transaction};
use sp_core::{H256, U256};
use starknet_api::api_core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};

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

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let transaction =
            Transaction { version: 1_u8, sender_address: contract_address_bytes, ..Transaction::default() };

        assert_err!(Starknet::add_invoke_transaction(none_origin, transaction), Error::<Test>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_invalid_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            0_u8,
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
                    U256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // Contract address
                    U256::from_str("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), // Selector
                    U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), // Length
                    U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), // Value
                ],
                contract_address_bytes,
                contract_address_bytes
            ),
            None,
        );

        assert_err!(Starknet::add_invoke_transaction(none_origin, transaction), Error::<Test>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a919";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
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
                    U256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // Contract address
                    U256::from_str("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), // Selector
                    U256::one(), // Length
                    U256::from(25), // Value
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );

        let tx =
            Message {
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

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a919";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
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

                    U256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // Contract address
                    U256::from_str("0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588").unwrap(), // Selector
                    U256::zero(),
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );
        assert_ok!(Starknet::add_invoke_transaction(none_origin, transaction));

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

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a919";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
                    // Constructor calldata
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );

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

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a919";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
                    // Constructor calldata
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );

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

        // TODO: Compute address from salt/hash/calldata/deployer
        let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "0334e1e4d148a789fb44367eff869a6330693037983ba6fd2291b2be1249e15a";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
                    // Constructor calldata
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );

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

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        let account_class = ContractClassWrapper::from(get_contract_class(ACCOUNT_CONTRACT_PATH));

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                contract_address_bytes,
                contract_address_bytes,
            ),
            Some(account_class),
            None,
        );

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

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        let account_class = ContractClassWrapper::from(get_contract_class(ACCOUNT_CONTRACT_PATH));

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                contract_address_bytes,
                contract_address_bytes,
            ),
            Some(account_class),
            None,
        );

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

        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );

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
        let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";
        let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

        let target_contract_address =
            U256::from_str("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        // test_storage_read_write
        let target_selector =
            U256::from_str("0x03b097c62d3e4b85742aadd0dfb823f96134b886ec13bda57b68faf86f294d97").unwrap();
        let storage_var_selector = U256::from(25);
        let storage_var_val = U256::one();

        let transaction = Transaction::new(
            1_u8,
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
                    target_contract_address,
                    target_selector,
                    U256::from(2),
                    storage_var_selector,
                    storage_var_val,
                ],
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            None,
        );
        let mut contract_address_bytes = [0_u8; 32];
        target_contract_address.to_big_endian(&mut contract_address_bytes);
        let mut storage_var_selector_bytes = [0_u8; 32];
        storage_var_selector.to_big_endian(&mut storage_var_selector_bytes);
        assert_ok!(Starknet::add_invoke_transaction(none_origin, transaction));
        assert_eq!(
            Starknet::storage((contract_address_bytes, H256::from_slice(&storage_var_selector_bytes))),
            U256::one()
        );
    });
}

#[test]
fn deploy_account_tx_with_salt() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let proxy_class_hash = "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
        let proxy_class_hash_bytes = <[u8; 32]>::from_hex(proxy_class_hash.strip_prefix("0x").unwrap()).unwrap();

        let account_class_hash = "0x033434ad846cdd5f23eb73ff09fe6fddd568284a0fb7d1be20ee482f044dabe2";

        let salt = StarkFelt::try_from("0x22c428661e51d30b5cb09cf3e9cae5565e4ef2c094b6c845f750c52fffb6f97").unwrap();

        let cd_raw = vec![
            account_class_hash,
            "0x79dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463",
            "0x2",
            "0x22c428661e51d30b5cb09cf3e9cae5565e4ef2c094b6c845f750c52fffb6f97",
            "0x0",
        ];
        let cd_felt = cd_raw.clone().into_iter().map(|x| StarkFelt::try_from(x).unwrap()).collect::<Vec<StarkFelt>>();
        let cd_u256 = cd_raw.clone().into_iter().map(|x| U256::from(x)).collect::<Vec<U256>>();

        let contract_addr = calculate_contract_address(
            ContractAddressSalt(salt),
            ClassHash(StarkFelt::new(proxy_class_hash_bytes).unwrap()),
            &Calldata(cd_felt.into()),
            ContractAddress::default(),
        )
        .unwrap();
        let contract_address_bytes = contract_addr.0.key().to_owned().0;

        let exp_addr = ContractAddress::try_from(
            StarkFelt::try_from("0x41b840332cda4bc8b157ae3feecfc95b9e057d8311a07b044e7d1521106313b").unwrap(),
        )
        .unwrap();
        assert_eq!(contract_addr, exp_addr);

        // Example TX: https://testnet.starkscan.co/tx/0x01ba94e9f0f919e23bdedc6a8e5e62bb1be9b257a65a72a1d4d236a93edc2a60
        let transaction = Transaction::new(
            U256::from(1),
            H256::default(),
            bounded_vec!(),
            bounded_vec!(),
            contract_address_bytes,
            U256::from(0),
            CallEntryPointWrapper::new(
                Some(proxy_class_hash_bytes),
                EntryPointTypeWrapper::External,
                None,
                BoundedVec::try_from(cd_u256).unwrap(),
                contract_address_bytes,
                contract_address_bytes,
            ),
            None,
            Some(H256::from(salt.0)),
        );

        assert_ok!(Starknet::add_deploy_account_transaction(none_origin, transaction));
    });
}
