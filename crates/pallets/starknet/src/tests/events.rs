use std::sync::Arc;

use blockifier::transaction::transactions::InvokeTransaction;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, InvokeTransactionV1, TransactionSignature};
use starknet_core::utils::get_selector_from_name;

use super::constants::{ETH_FEE_TOKEN_ADDRESS, MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS};
use super::mock::default_mock::*;
use super::mock::*;

const INNER_EVENT_EMITTING_CONTRACT_ADDRESS: &str =
    "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf";

#[test]
fn internal_and_external_events_are_emitted_in_the_right_order() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let chain_id = Starknet::chain_id();

        let emit_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap()));
        let inner_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(INNER_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap()));
        let fee_token_address = ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let emit_selector: StarkFelt = Felt252Wrapper::from(get_selector_from_name("emit_sandwich").unwrap()).into();

        let tx = InvokeTransactionV1 {
            sender_address: sender_account,
            calldata: Calldata(Arc::new(vec![
                emit_contract_address.0.0, // Token address
                emit_selector,
                StarkFelt::ZERO, // Calldata len
            ])),
            nonce: Nonce(StarkFelt::ZERO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
        };
        let tx_hash = tx.compute_hash(chain_id, false);
        let transaction = InvokeTransaction { tx: tx.into(), tx_hash, only_query: false };

        Starknet::invoke(RuntimeOrigin::none(), transaction.clone())
            .expect("emit sandwich transaction should not fail");

        let events = Starknet::tx_events(tx_hash);
        let event_emitters: Vec<ContractAddress> = events.iter().map(|event| event.from_address).collect();

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
