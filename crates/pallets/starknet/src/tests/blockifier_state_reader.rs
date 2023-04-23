use core::str::FromStr;

use blockifier::state::state_api::StateReader;
use blockifier::test_utils::{get_contract_class, ERC20_CONTRACT_PATH};
use frame_support::{assert_ok, bounded_vec, BoundedVec};
use hex::FromHex;
use mp_starknet::execution::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::Transaction;
use sp_core::{H256, U256};
use starknet_api::api_core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;

use super::mock::*;
use crate::state_reader::BlockifierStateReader;

#[test]
fn given_a_declared_contract_when_calling_trait_state_reader_get_contract_class_method_then_the_contract_class_is_returned()
 {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let contract_class = get_contract_class(ERC20_CONTRACT_PATH);
        let erc20_class = ContractClassWrapper::from(contract_class.clone());
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = Transaction {
            sender_address: account_addr,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                account_addr,
                account_addr,
            ),
            contract_class: Some(erc20_class),
            ..Transaction::default()
        };

        assert_ok!(Starknet::declare(none_origin, transaction));

        let mut state_reader = BlockifierStateReader::<Test>::new();
        let retrieved_contract_class =
            (*state_reader.get_contract_class(&ClassHash(StarkFelt(erc20_class_hash))).unwrap()).clone();

        assert!(retrieved_contract_class.abi.is_none());
        assert_eq!(retrieved_contract_class.program, contract_class.program);
        assert_eq!(retrieved_contract_class.entry_points_by_type, contract_class.entry_points_by_type);
    });
}

#[test]
fn given_a_deployed_contract_when_state_reader_get_class_hash_at_method_is_called_the_correct_class_hash_is_returned() {
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
        let mut state_reader = BlockifierStateReader::<Test>::new();
        let class_hash = state_reader.get_class_hash_at(ContractAddress(PatriciaKey(StarkFelt(test_addr)))).unwrap();
        assert_eq!(class_hash, ClassHash(StarkFelt(account_class_hash)));
    });
}
