use core::str::FromStr;

use blockifier::execution::contract_class::ContractClass;
use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH, ERC20_CONTRACT_PATH};
use frame_support::{assert_err, assert_ok, bounded_vec, debug, BoundedVec};
use hex::FromHex;
use hexlit::hex;
use lazy_static::lazy_static;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::execution::{
    CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper,
};
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{EventWrapper, Transaction, TxType};
use sp_core::{H256, U256};
use sp_runtime::transaction_validity::InvalidTransaction::Payment;
use sp_runtime::transaction_validity::TransactionValidityError::Invalid;
use sp_runtime::DispatchError;

use crate::mock::*;
use crate::types::Message;
use crate::{Error, Event, SEQUENCER_ADDRESS};

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
            sequencer_address: SEQUENCER_ADDRESS,
            ..StarknetHeader::default()
        };

        pretty_assertions::assert_eq!(*current_block.header(), expected_current_block);
        pretty_assertions::assert_eq!(current_block.transactions_hashes().len(), 1);
        pretty_assertions::assert_eq!(
            current_block.transactions_hashes().get(0).unwrap(),
            &H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
        );
        debug(&current_block);
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

        let json_content: &str = include_str!("../../../../resources/transactions/invoke_invalid_version.json");
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

        let json_content: &str = include_str!("../../../../resources/transactions/invoke.json");
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

        let json_content: &str = include_str!("../../../../resources/transactions/invoke_emit_event.json");
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

        let json_content: &str = include_str!("../../../../resources/transactions/storage_read_write.json");
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
                BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>()).unwrap(),
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
                BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>()).unwrap(),
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

        transaction.contract_class = Some(erc20_class);

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn given_balance_on_account_then_transfer_fees_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let from = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15];
        let to = Starknet::current_block().header().sequencer_address;
        let amount = 100;
        let token_address = Starknet::fee_token_address();

        assert_ok!(Starknet::transfer_fees(from, to, amount));
        // Check that balance is deducted from sn account
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap(),
            )),
            U256::from(u128::MAX) - amount
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap(),
            )),
            U256::from(u128::MAX)
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x02) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x01d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f").unwrap(),
            )),
            U256::from(amount)
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x02) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x01d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f21470").unwrap(),
            )),
            U256::zero()
        );
        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec!(
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ),
                data: bounded_vec!(
                    // From
                    H256::from_slice(&from),
                    // To
                    H256::from_slice(&SEQUENCER_ADDRESS),
                    // Amount low
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000064").unwrap(),
                    // Amount High
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                ),
                from_address: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 170,
                ],
            })
            .into(),
        );
    })
}
#[test]
fn given_no_balance_on_account_then_transfer_fees_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let from = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let to = Starknet::current_block().header().sequencer_address;
        let amount = 100;

        assert_err!(Starknet::transfer_fees(from, to, amount), Invalid(Payment));
        // Check that balance is not deducted from sn account
        assert_eq!(
            Starknet::storage((
                <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap(), // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x01) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap(),
            )),
            U256::from(u128::MAX)
        );
        assert_eq!(
            Starknet::storage((
                <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap(), // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x01) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap(),
            )),
            U256::from(u128::MAX)
        )
    })
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

        assert_ok!(Starknet::set_fee_token_address(root_origin, new_fee_token_address));
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

#[test]
fn given_erc20_transfer_when_invoke_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(1);
        let origin = RuntimeOrigin::none();
        let (sender_account, _, _) = account_helper(TEST_ACCOUNT_SALT);
        // Declare ERC20 contract
        declare_erc20(origin.clone(), sender_account);
        // Deploy ERC20 contract
        deploy_erc20(origin.clone(), sender_account);
        // TODO: use dynamic values to craft invoke transaction
        // Transfer some token
        invoke_transfer_erc20(origin, sender_account);
        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000f").unwrap(),
                    H256::from_str("0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap(),
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                ),
                from_address: H256::from_str("0x0074c41dd9ba722396796cba415f8a742d671eb872371c96ce1ce6016fd0f2bb")
                    .unwrap()
                    .to_fixed_bytes(),
            })
            .into(),
        );
    })
}

/// Helper function to declare ERC20 contract.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn declare_erc20(origin: RuntimeOrigin, sender_account: ContractAddressWrapper) {
    let declare_transaction = Transaction {
        sender_address: sender_account,
        call_entrypoint: CallEntryPointWrapper::new(
            Some(ERC20_CLASS_HASH),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![],
            sender_account,
            sender_account,
        ),
        contract_class: Some(ERC20_CONTRACT_CLASS.clone()),
        ..Transaction::default()
    };
    assert_ok!(Starknet::declare(origin, declare_transaction));
}

/// Helper function to deploy ERC20 contract.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn deploy_erc20(origin: RuntimeOrigin, _sender_account: ContractAddressWrapper) {
    let deploy_transaction = transaction_from_json(
        include_str!("../../../../resources/transactions/deploy_erc20.json"),
        include_bytes!("../../../../resources/account/account.json"),
    )
    .unwrap();
    assert_ok!(Starknet::invoke(origin, deploy_transaction));
}

/// Helper function to mint some tokens.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn invoke_transfer_erc20(origin: RuntimeOrigin, _sender_account: ContractAddressWrapper) {
    let erc20_mint_tx_json: &str = include_str!("../../../../resources/transactions/invoke_erc20_transfer.json");
    let erc20_mint_tx = transaction_from_json(erc20_mint_tx_json, &[]).expect("Failed to create Transaction from JSON");
    assert_ok!(Starknet::invoke(origin, erc20_mint_tx));
}

lazy_static! {
    static ref ERC20_CONTRACT_CLASS: ContractClassWrapper =
        get_contract_class_wrapper(include_bytes!("../../../../resources/erc20/erc20.json"));
}
const ERC20_CLASS_HASH: [u8; 32] = hex!("01d1aacf8f874c4a865b974236419a46383a5161925626e9053202d8e87257e9");

fn get_contract_class_wrapper(contract_content: &'static [u8]) -> ContractClassWrapper {
    let contract_class: ContractClass =
        serde_json::from_slice(contract_content).expect("File must contain the content of a compiled contract.");
    ContractClassWrapper::from(contract_class)
}
