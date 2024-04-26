use frame_support::assert_ok;
use mp_felt::Felt252Wrapper;
use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

use super::constants::FEE_TOKEN_ADDRESS;
use super::mock::*;
use super::utils::{
    build_get_balance_contract_call, build_transfer_invoke_transaction, BuildTransferInvokeTransaction,
};
use crate::tests::mock::setup_mock::default_mock::*;

#[test]
fn given_default_runtime_with_fees_enabled_txn_deducts_fee_token() {
    new_test_ext::<default_mock::MockRuntime>().execute_with(|| {
        default_mock::basic_test_setup(2);
        let origin = default_mock::RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let (initial_balance_low, initial_balance_high) = get_balance_default_mock(address);

        // transfer to zero fee token so that the only change in balance can happen because of fees
        assert_ok!(default_mock::Starknet::invoke(origin, build_invoke_transaction(chain_id, address)));
        let (final_balance_low, final_balance_high) = get_balance_default_mock(address);

        // Check that the balance has changed because fees is reduced
        assert!(initial_balance_low > final_balance_low);
        pretty_assertions::assert_eq!(initial_balance_high, final_balance_high);
    });
}

#[test]
fn given_default_runtime_with_fees_disabled_txn_does_not_deduct_fee_token() {
    new_test_ext::<fees_disabled_mock::MockRuntime>().execute_with(|| {
        fees_disabled_mock::basic_test_setup(2);
        let origin = fees_disabled_mock::RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let (initial_balance_low, initial_balance_high) = get_balance_fees_disabled_mock(address);

        // transfer to zero fee token so that the only change in balance can happen because of fees
        assert_ok!(fees_disabled_mock::Starknet::invoke(origin, build_invoke_transaction(chain_id, address)));
        let (final_balance_low, final_balance_high) = get_balance_fees_disabled_mock(address);

        // Check that the balance hasn't changed
        pretty_assertions::assert_eq!(initial_balance_low, final_balance_low);
        pretty_assertions::assert_eq!(initial_balance_high, final_balance_high);
    });
}

fn build_invoke_transaction(
    chain_id: Felt252Wrapper,
    address: ContractAddress,
) -> blockifier::transaction::transactions::InvokeTransaction {
    build_transfer_invoke_transaction(
        chain_id,
        BuildTransferInvokeTransaction {
            sender_address: address,
            token_address: ContractAddress(PatriciaKey(StarkFelt::try_from(FEE_TOKEN_ADDRESS).unwrap())),
            recipient: address,
            amount_low: StarkFelt::ZERO,
            amount_high: StarkFelt::ZERO,
            nonce: Nonce(StarkFelt::ZERO),
        },
    )
}

fn get_balance_default_mock(account_address: ContractAddress) -> (Felt252Wrapper, Felt252Wrapper) {
    let (selector, calldata) = build_get_balance_call(account_address);
    let result = default_mock::Starknet::call_contract(
        ContractAddress(PatriciaKey(StarkFelt::try_from(FEE_TOKEN_ADDRESS).unwrap())),
        selector,
        calldata,
    )
    .unwrap();
    (result[0], result[1])
}

fn get_balance_fees_disabled_mock(account_address: ContractAddress) -> (Felt252Wrapper, Felt252Wrapper) {
    let (selector, calldata) = build_get_balance_call(account_address);
    let result = fees_disabled_mock::Starknet::call_contract(
        ContractAddress(PatriciaKey(StarkFelt::try_from(FEE_TOKEN_ADDRESS).unwrap())),
        selector,
        calldata,
    )
    .unwrap();
    (result[0], result[1])
}

fn build_get_balance_call(account_address: ContractAddress) -> (EntryPointSelector, Calldata) {
    build_get_balance_contract_call(account_address.0.0)
}
