use frame_support::assert_ok;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::{DeclareTransactionV1, DeployAccountTransaction, InvokeTransactionV1};
use starknet_api::api_core::EthAddress;
use starknet_api::transaction::{L2ToL1Payload, MessageToL1, TransactionHash};

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::get_contract_class;
use crate::Config;

// NoValidateAccount (Cairo 0)
const DEPLOY_CONTRACT_SELECTOR: &str = "0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8";

// Generate: starkli class-hash ./cairo-contracts/build/send_message.json
const SEND_MESSAGE_CLASS_HASH: &str = "0x024e12108da2b3d3c80c7bc99aacdec97e445199c5a8ded7435bf77ce4507631";
// const SEND_MESSAGE_TO_L2_SELECTOR: &str =
// "0x38fb0bf48fe489ae23d0a1d7f2b7195ec0b94bfeb2f408b13bfd943d8410d72";
const SEND_MESSAGE_TO_L1_SELECTOR: &str = "0x9139dbd19ca9654d773cd88f31af4c8d583beecc3362fb986dccfef5cf134f";

// Troubleshooting notes:
// Add println! to the necessary runtime method (e.g. print CallInfo structs or extended blockifier
// message), then run:
// cargo test --package pallet-starknet send_message -- --nocapture

#[test]
fn messages_to_l1_are_stored() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_address: Felt252Wrapper =
            get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate)).into();
        let contract_class = get_contract_class("send_message.json", 0);
        let class_hash = Felt252Wrapper::from_hex_be(SEND_MESSAGE_CLASS_HASH).unwrap();

        let declare_tx = DeclareTransactionV1 {
            sender_address,
            class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx.into(), contract_class));

        let salt = Felt252Wrapper::ZERO;
        let contract_address: Felt252Wrapper =
            DeployAccountTransaction::calculate_contract_address(salt.into(), class_hash.into(), &[]).into();

        let deploy_tx = InvokeTransactionV1 {
            sender_address,
            calldata: vec![
                sender_address,
                Felt252Wrapper::from_hex_be(DEPLOY_CONTRACT_SELECTOR).unwrap(),
                Felt252Wrapper::from(3u128), // Calldata len
                class_hash,
                salt,
                Felt252Wrapper::ZERO, // Constructor calldata len (no explicit constructor declared)
            ],
            nonce: Felt252Wrapper::ONE,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), deploy_tx.into()));

        let invoke_tx = InvokeTransactionV1 {
            sender_address,
            calldata: vec![
                contract_address,
                Felt252Wrapper::from_hex_be(SEND_MESSAGE_TO_L1_SELECTOR).unwrap(),
                Felt252Wrapper::from(3u128), // Calldata len
                Felt252Wrapper::ZERO,        // to_address
                Felt252Wrapper::ONE,         // payload_len
                Felt252Wrapper::TWO,         // payload
            ],
            nonce: Felt252Wrapper::TWO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), invoke_tx.clone().into()));

        let chain_id = Starknet::chain_id();
        let tx_hash = invoke_tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        let messages = Starknet::tx_messages(TransactionHash::from(tx_hash));

        assert_eq!(1, messages.len());
        pretty_assertions::assert_eq!(
            messages[0],
            MessageToL1 {
                from_address: contract_address.into(),
                to_address: EthAddress([0u8; 20].into()),
                payload: L2ToL1Payload(vec![Felt252Wrapper::TWO.into()])
            }
        );
    });
}
