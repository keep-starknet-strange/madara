use frame_support::{assert_ok, bounded_vec};
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::InvokeTransaction;

use super::constants::FEE_TOKEN_ADDRESS;
use super::mock::fees_disabled_mock::{basic_test_setup_fees_disabled, new_test_ext_with_fees_disabled};
use super::mock::*;
use crate::tests::mock::fees_disabled_mock::{
    MockFeesDisabledRuntime, RuntimeOrigin as FeesDisabledRuntimeOrigin, Starknet as FeesDisabledStarknet,
};

#[test]
fn given_default_runtime_with_fees_enabled_txn_deducts_fee_token() {
    new_test_ext().execute_with(|| {
        basic_test_setup_fees_disabled::<MockRuntime>(2);
        let origin = RuntimeOrigin::none();

        let address = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let (initial_balance_low, initial_balance_high) = get_balance(address);

        // transfer to zero fee token so that the only change in balance can happen because of fees
        assert_ok!(Starknet::invoke(origin, build_invoke_transaction(address)));
        let (final_balance_low, final_balance_high) = get_balance(address);

        // Check that the balance has changed because fees is reduced
        assert!(initial_balance_low > final_balance_low);
        pretty_assertions::assert_eq!(initial_balance_high, final_balance_high);
    });
}

#[test]
fn given_default_runtime_with_fees_disabled_txn_does_not_deduct_fee_token() {
    new_test_ext_with_fees_disabled().execute_with(|| {
        basic_test_setup_fees_disabled::<MockFeesDisabledRuntime>(2);
        let origin = FeesDisabledRuntimeOrigin::none();

        let address = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let (initial_balance_low, initial_balance_high) = get_balance(address);

        // transfer to zero fee token so that the only change in balance can happen because of fees
        assert_ok!(FeesDisabledStarknet::invoke(origin, build_invoke_transaction(address)));
        let (final_balance_low, final_balance_high) = get_balance(address);

        // Check that the balance hasn't changed
        pretty_assertions::assert_eq!(initial_balance_low, final_balance_low);
        pretty_assertions::assert_eq!(initial_balance_high, final_balance_high);
    });
}

fn build_invoke_transaction(address: ContractAddressWrapper) -> InvokeTransaction {
    InvokeTransaction {
        version: 1,
        sender_address: address,
        calldata: bounded_vec![
            Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(), // Token address
            Felt252Wrapper::from_hex_be("0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e").unwrap(), /* transfer selector */
            Felt252Wrapper::THREE, // Calldata len
            address,               // recipient
            Felt252Wrapper::ZERO,  // initial supply low
            Felt252Wrapper::ZERO,  // initial supply high
        ],
        nonce: Felt252Wrapper::ZERO,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        is_query: false,
    }
}

fn get_balance(account_address: ContractAddressWrapper) -> (Felt252Wrapper, Felt252Wrapper) {
    let balance_of_selector =
        Felt252Wrapper::from_hex_be("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap();
    let calldata = bounded_vec![
        account_address // owner address
    ];
    let res =
        Starknet::call_contract(Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(), balance_of_selector, calldata)
            .unwrap();
    (res[0], res[1])
}
