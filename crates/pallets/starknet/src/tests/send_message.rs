use std::sync::Arc;

use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::transactions::InvokeTransaction;
use frame_support::assert_ok;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{calculate_contract_address, ClassHash, EthAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeclareTransactionV0V1, Fee, InvokeTransactionV1, L2ToL1Payload, MessageToL1,
    TransactionSignature,
};

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::get_contract_class;

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

        let chain_id = Starknet::chain_id();
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let contract_class = get_contract_class("send_message.json", 0);
        let class_hash = ClassHash(StarkFelt::try_from(SEND_MESSAGE_CLASS_HASH).unwrap());

        let declare_tx = starknet_api::transaction::DeclareTransaction::V1(DeclareTransactionV0V1 {
            sender_address,
            class_hash,
            nonce: Nonce(StarkFelt::ZERO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature(vec![]),
        });
        let tx_hash = declare_tx.compute_hash(chain_id, false);
        let declare_tx = blockifier::transaction::transactions::DeclareTransaction::new(
            declare_tx,
            tx_hash,
            ClassInfo::new(&contract_class, 0, 100).unwrap(),
        )
        .unwrap();

        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx));

        let salt = ContractAddressSalt(StarkFelt::ZERO);
        let contract_address =
            calculate_contract_address(salt, class_hash, &Calldata(Default::default()), Default::default()).unwrap();

        let deploy_tx = InvokeTransactionV1 {
            sender_address,
            calldata: Calldata(Arc::new(vec![
                sender_address.0.0,
                StarkFelt::try_from(DEPLOY_CONTRACT_SELECTOR).unwrap(),
                StarkFelt::from(3u128), // Calldata len
                class_hash.0,
                salt.0,
                StarkFelt::ZERO, // Constructor calldata len (no explicit constructor declared)
            ])),
            nonce: Nonce(StarkFelt::ONE),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature(vec![]),
        };

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), deploy_tx.into()));

        let tx = InvokeTransactionV1 {
            sender_address,
            calldata: Calldata(Arc::new(vec![
                contract_address.0.0,
                StarkFelt::try_from(SEND_MESSAGE_TO_L1_SELECTOR).unwrap(),
                StarkFelt::from(3u128), // Calldata len
                StarkFelt::ZERO,        // to_address
                StarkFelt::ONE,         // payload_len
                StarkFelt::TWO,         // payload
            ])),
            nonce: Nonce(StarkFelt::TWO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature(vec![]),
        };

        let chain_id = Starknet::chain_id();
        let tx_hash = tx.compute_hash(chain_id, false);

        let transaction = InvokeTransaction { tx: tx.into(), tx_hash, only_query: false };
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction));

        let messages = Starknet::tx_messages(tx_hash);

        assert_eq!(1, messages.len());
        pretty_assertions::assert_eq!(
            messages[0],
            MessageToL1 {
                from_address: contract_address,
                to_address: EthAddress([0u8; 20].into()),
                payload: L2ToL1Payload(vec![Felt252Wrapper::TWO.into()])
            }
        );
    });
}
