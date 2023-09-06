use frame_support::bounded_vec;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::InvokeTransaction;
use starknet_core::utils::get_selector_from_name;

use super::constants::{FEE_TOKEN_ADDRESS, MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS};
use super::mock::default_mock::*;
use super::mock::*;

const INNER_EVENT_EMITTING_CONTRACT_ADDRESS: &str =
    "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf";

#[test]
fn internal_and_external_events_are_emitted_in_the_right_order() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let emit_contract_address = Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();
        let inner_contract_address = Felt252Wrapper::from_hex_be(INNER_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();
        let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let emit_selector = Felt252Wrapper::from(get_selector_from_name("emit_sandwich").unwrap());

        let emit_event_transaction = InvokeTransaction {
            version: 1,
            sender_address: sender_account,
            calldata: bounded_vec![
                emit_contract_address, // Token address
                emit_selector,
                Felt252Wrapper::ZERO, // Calldata len
            ],
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        let none_origin = RuntimeOrigin::none();
        Starknet::invoke(none_origin, emit_event_transaction).expect("emit sandwich transaction failed");

        let pending = Starknet::pending();
        let receipt = &pending.get(0).unwrap().1;
        let event_emitters: Vec<Felt252Wrapper> = receipt.events.iter().map(|event| event.from_address).collect();

        pretty_assertions::assert_eq!(
            event_emitters,
            vec![
                emit_contract_address,  // internal
                inner_contract_address, // external
                emit_contract_address,  // internal
                inner_contract_address, // external
                emit_contract_address,  // internal
                fee_token_address       // fee transfer
            ]
        );
    });
}
