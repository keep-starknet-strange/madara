use frame_support::assert_ok;
use mp_felt::Felt252Wrapper;
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;

use super::mock::{new_test_ext, no_nonce_validation_mock};
use crate::tests::constants::SALT;
use crate::tests::mock::no_nonce_validation_mock::{basic_test_setup, RuntimeOrigin, Starknet};
use crate::tests::mock::{get_account_address, AccountType, AccountTypeV0Inner};
use crate::tests::utils::get_contract_class;
use crate::tests::{get_declare_dummy, get_deploy_account_dummy, get_invoke_dummy, set_infinite_tokens};

#[test]
fn given_invoke_tx_with_invalid_nonce_then_it_does_nothing() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction = get_invoke_dummy(Felt252Wrapper::MAX);
        let sender_address = transaction.sender_address;

        assert_ok!(Starknet::invoke(none_origin, transaction.into()));

        // check nonce is still 0
        let nonce = Starknet::nonce(ContractAddress::from(sender_address));
        assert_eq!(nonce, Nonce(StarkFelt::from(0u128)));
    });
}

#[test]
fn given_declare_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::MAX, AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        let erc20_class_hash = *transaction.class_hash();
        let sender_address = *transaction.sender_address();

        let contract_class = get_contract_class("ERC20.json", 0);

        assert_ok!(Starknet::declare(none_origin, transaction.clone(), contract_class.clone()));
        assert_eq!(Starknet::contract_class_by_class_hash(ClassHash::from(erc20_class_hash)).unwrap(), contract_class);

        // check nonce is still 0
        let nonce = Starknet::nonce(ContractAddress::from(sender_address));
        assert_eq!(nonce, Nonce(StarkFelt::from(0u128)));
    });
}

#[test]
fn given_deploy_account_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let transaction =
            get_deploy_account_dummy(Felt252Wrapper::MAX, *SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let account_class_hash = transaction.class_hash;

        let address = get_account_address(Some(*SALT), AccountType::V0(AccountTypeV0Inner::NoValidate));
        set_infinite_tokens::<no_nonce_validation_mock::MockRuntime>(&address);

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash.into());

        // check nonce is still 0
        let nonce = Starknet::nonce(address);
        assert_eq!(nonce, Nonce(StarkFelt::from(0u128)));
    });
}
