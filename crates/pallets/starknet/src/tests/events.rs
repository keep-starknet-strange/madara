use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::InvokeTransactionV1;
use starknet_api::transaction::TransactionHash;
use starknet_core::utils::get_selector_from_name;

use super::constants::{FEE_TOKEN_ADDRESS, MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS};
use super::mock::default_mock::*;
use super::mock::*;
use crate::Config;

const INNER_EVENT_EMITTING_CONTRACT_ADDRESS: &str =
    "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf";

#[test]
fn internal_and_external_events_are_emitted_in_the_right_order() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let emit_contract_address = Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();
        let inner_contract_address = Felt252Wrapper::from_hex_be(INNER_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();
        let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

        let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let emit_selector = Felt252Wrapper::from(get_selector_from_name("emit_sandwich").unwrap());

        let emit_event_transaction = InvokeTransactionV1 {
            sender_address: sender_account.into(),
            calldata: vec![
                emit_contract_address, // Token address
                emit_selector,
                Felt252Wrapper::ZERO, // Calldata len
            ],
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        let none_origin = RuntimeOrigin::none();
        Starknet::invoke(none_origin, emit_event_transaction.clone().into())
            .expect("emit sandwich transaction should not fail");

        let chain_id = Starknet::chain_id();
        let tx_hash = emit_event_transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        let events = Starknet::tx_events(TransactionHash::from(tx_hash));
        let event_emitters: Vec<Felt252Wrapper> =
            events.iter().map(|event| Felt252Wrapper::from(event.from_address)).collect();

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
