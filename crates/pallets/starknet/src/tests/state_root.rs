use frame_support::{assert_err, assert_ok, bounded_vec};
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::DeclareTransaction;

use super::mock::state_root_mock::new_test_ext_with_state_root;
use super::mock::*;
use super::utils::get_contract_class;
use crate::Error;

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext_with_state_root().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(AccountType::NoValidate);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("ERC20.json")).unwrap();
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            contract_class: erc20_class,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::ClassHashAlreadyDeclared);

        // State root should have been updated accordingly
    });
}
