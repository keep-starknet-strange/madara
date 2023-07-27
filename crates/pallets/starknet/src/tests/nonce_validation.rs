use frame_support::assert_ok;
use mp_starknet::crypto::commitment::calculate_declare_tx_hash;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::InvokeTransaction;

use super::mock::{new_test_ext, no_nonce_validation_mock};
use crate::tests::constants::SALT;
use crate::tests::mock::no_nonce_validation_mock::{basic_test_setup, RuntimeOrigin, Starknet};
use crate::tests::mock::{account_helper, AccountType, AccountTypeV0Inner};
use crate::tests::utils::get_contract_class;
use crate::tests::{
    get_declare_dummy, get_deploy_account_dummy, get_invoke_dummy, set_infinite_tokens, sign_message_hash,
};

#[test]
fn given_invoke_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction: InvokeTransaction = get_invoke_dummy().into();
        transaction.nonce = Felt252Wrapper::MAX; // modify nonce to be invalid

        assert_ok!(Starknet::invoke(none_origin, transaction.clone()));

        // check nonce is still 0
        let nonce = Starknet::nonce(transaction.sender_address);
        assert_eq!(nonce, Felt252Wrapper::from(0u8));
    });
}

#[test]
fn given_declare_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        transaction.nonce = Felt252Wrapper::MAX; // modify nonce to be invalid
        let erc20_class_hash = transaction.class_hash;

        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_declare_tx_hash(transaction.clone(), chain_id);
        transaction.signature = sign_message_hash(transaction_hash);

        assert_ok!(Starknet::declare(none_origin, transaction.clone()));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            get_contract_class("ERC20.json", 0)
        );

        // check nonce is still 0
        let nonce = Starknet::nonce(transaction.sender_address);
        assert_eq!(nonce, Felt252Wrapper::from(0u8));
    });
}

#[test]
fn given_deploy_account_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));
        transaction.nonce = Felt252Wrapper::MAX; // modify nonce to be invalid
        let account_class_hash = transaction.account_class_hash;

        let (address, _, _) = account_helper(*SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));
        set_infinite_tokens::<no_nonce_validation_mock::MockRuntime>(address);

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address).unwrap(), account_class_hash);

        // check nonce is still 0
        let nonce = Starknet::nonce(address);
        assert_eq!(nonce, Felt252Wrapper::from(0u8));
    });
}
