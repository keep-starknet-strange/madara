use blockifier::execution::contract_class::ContractClass;
use frame_support::assert_ok;
use lazy_static::lazy_static;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::InvokeTransactionV1;
use starknet_api::api_core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Event as StarknetEvent, EventContent, EventData, EventKey, TransactionHash};
use starknet_core::utils::get_selector_from_name;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::constants::TOKEN_CONTRACT_CLASS_HASH;
use crate::tests::utils::{build_transfer_invoke_transaction, get_contract_class};
use crate::types::BuildTransferInvokeTransaction;
use crate::Config;

lazy_static! {
    static ref ERC20_CONTRACT_CLASS: ContractClass = get_contract_class("ERC20.json", 0);
}

#[test]
fn given_erc20_transfer_when_invoke_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(1);
        let origin = RuntimeOrigin::none();
        let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let felt_252_sender_account = sender_account.into();
        // ERC20 is already declared for the fees.
        // Deploy ERC20 contract
        let deploy_transaction = InvokeTransactionV1 {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            sender_address: felt_252_sender_account,
            calldata: vec![
                felt_252_sender_account, // Simple contract address
                Felt252Wrapper::from_hex_be("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8")
                    .unwrap(), // deploy_contract selector
                Felt252Wrapper::from_hex_be("0x9").unwrap(), // Calldata len
                Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
                Felt252Wrapper::ONE,     // Contract address salt
                Felt252Wrapper::from_hex_be("0x6").unwrap(), // Constructor_calldata_len
                Felt252Wrapper::from_hex_be("0xA").unwrap(), // Name
                Felt252Wrapper::from_hex_be("0x1").unwrap(), // Symbol
                Felt252Wrapper::from_hex_be("0x2").unwrap(), // Decimals
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply low
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply high
                felt_252_sender_account, // recipient
            ],
            offset_version: false,
        };

        let expected_erc20_address =
            StarkFelt::try_from("0x00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap();

        let chain_id = Starknet::chain_id();
        let tx_hash = deploy_transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);

        assert_ok!(Starknet::invoke(origin.clone(), deploy_transaction.into()));

        let events: Vec<StarknetEvent> = Starknet::tx_events(TransactionHash::from(tx_hash));
        // Expected events:
        // ERC20 -> Transfer
        // NoValidateAccount -> ContractDeployed
        // FeeToken -> Transfer

        // Check transaction event (deployment)
        pretty_assertions::assert_eq!(
            StarknetEvent {
                content: EventContent {
                    keys: vec![EventKey(
                        StarkFelt::try_from("0x026b160f10156dea0639bec90696772c640b9706a47f5b8c52ea1abe5858b34d")
                            .unwrap()
                    )],
                    data: EventData(vec![
                        expected_erc20_address, // Contract address
                        StarkFelt::from(0u128), /* Deployer (always 0 with this
                                                 * account contract) */
                        StarkFelt::try_from(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
                        StarkFelt::try_from("0x0000000000000000000000000000000000000000000000000000000000000006")
                            .unwrap(), // Constructor calldata len
                        StarkFelt::try_from("0x000000000000000000000000000000000000000000000000000000000000000a")
                            .unwrap(), // Name
                        StarkFelt::try_from("0x0000000000000000000000000000000000000000000000000000000000000001")
                            .unwrap(), // Symbol
                        StarkFelt::try_from("0x0000000000000000000000000000000000000000000000000000000000000002")
                            .unwrap(), // Decimals
                        StarkFelt::try_from("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff")
                            .unwrap(), // Initial supply low
                        StarkFelt::try_from("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff")
                            .unwrap(), // Initial supply high
                        StarkFelt::try_from("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0")
                            .unwrap(), // Recipient
                        StarkFelt::try_from("0x0000000000000000000000000000000000000000000000000000000000000001")
                            .unwrap(), // Salt
                    ]),
                },
                from_address: sender_account,
            },
            events[1],
        );
        let expected_fee_transfer_event = StarknetEvent {
            content: EventContent {
                keys: vec![EventKey(
                    Felt252Wrapper::from(get_selector_from_name(mp_fee::TRANSFER_SELECTOR_NAME).unwrap()).into(),
                )],
                data: EventData(vec![
                    sender_account.0 .0, // From
                    StarkFelt::try_from("0x000000000000000000000000000000000000000000000000000000000000dead").unwrap(), // Sequencer address
                    StarkFelt::try_from("0x00000000000000000000000000000000000000000000000000000000000197a8").unwrap(), // Amount low
                    StarkFelt::from(0u128), // Amount high
                ]),
            },
            from_address: Starknet::fee_token_address(),
        };
        // Check fee transfer event
        pretty_assertions::assert_eq!(
            expected_fee_transfer_event,
            events.last().unwrap().clone()
        );
        // TODO: use dynamic values to craft invoke transaction
        // Transfer some token
        let transfer_transaction = build_transfer_invoke_transaction(BuildTransferInvokeTransaction {
            sender_address: felt_252_sender_account,
            token_address: expected_erc20_address.into(),
            recipient: Felt252Wrapper::from(16u128),
            amount_low: Felt252Wrapper::from(15u128),
            amount_high: Felt252Wrapper::ZERO,
            nonce: Felt252Wrapper::ONE,
        });

        let chain_id = Starknet::chain_id();
        let tx_hash = transfer_transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);

        // Also asserts that the deployment has been saved.
        assert_ok!(Starknet::invoke(origin, transfer_transaction));
        pretty_assertions::assert_eq!(
            Starknet::storage((
                ContractAddress(PatriciaKey(expected_erc20_address)),
                StorageKey(PatriciaKey(
                    StarkFelt::try_from("03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb403f").unwrap()
                ))
            )),
            StarkFelt::try_from("ffffffffffffffffffffffffffffff0").unwrap()
        );
        pretty_assertions::assert_eq!(
            Starknet::storage((
                ContractAddress(PatriciaKey(expected_erc20_address)),
                StorageKey(PatriciaKey(
                    StarkFelt::try_from("03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb4040").unwrap()
                ))
            )),
            StarkFelt::try_from("fffffffffffffffffffffffffffffff").unwrap()
        );
        pretty_assertions::assert_eq!(
            Starknet::storage((
                ContractAddress(PatriciaKey(expected_erc20_address)),
                StorageKey(PatriciaKey(
                    StarkFelt::try_from("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73e").unwrap()
                ))
            )),
            StarkFelt::from(15u128)
        );
        pretty_assertions::assert_eq!(
            Starknet::storage((
                ContractAddress(PatriciaKey(expected_erc20_address)),
                StorageKey(PatriciaKey(
                    StarkFelt::try_from("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73f").unwrap()
                ))
            )),
            StarkFelt::from(0u128)
        );

        let events: Vec<StarknetEvent> = Starknet::tx_events(TransactionHash::from(tx_hash));
        // Expected events: (added on top of the past ones)
        // ERC20 -> Transfer
        // FeeToken -> Transfer

        // Check regular event.
        let expected_event = StarknetEvent {
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from(Felt252Wrapper::from(get_selector_from_name(mp_fee::TRANSFER_SELECTOR_NAME).unwrap())).unwrap(),
                )],
                data: EventData(vec![
                    StarkFelt::try_from("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap(), // From
                    StarkFelt::try_from("0x10").unwrap(), // To
                    StarkFelt::try_from("0xF").unwrap(),  // Amount low
                    StarkFelt::from(0u128),               // Amount high
                ]),
            },
            from_address: ContractAddress(PatriciaKey(
                StarkFelt::try_from("0x00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap(),
            )),
        };

        pretty_assertions::assert_eq!(expected_event, events[events.len() - 2]);
        // Check fee transfer.
        let expected_fee_transfer_event = StarknetEvent {
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from(Felt252Wrapper::from(get_selector_from_name(mp_fee::TRANSFER_SELECTOR_NAME).unwrap())).unwrap(),
                )],
                data: EventData(vec![
                    sender_account.0 .0,                    // From
                    StarkFelt::try_from("0xdead").unwrap(), // Sequencer address
                    StarkFelt::try_from("0xf014").unwrap(), // Amount low
                    StarkFelt::from(0u128),                 // Amount high
                ]),
            },
            from_address: Starknet::fee_token_address(),
        };
        pretty_assertions::assert_eq!(
            expected_fee_transfer_event,
            events.last().unwrap().clone()
        );
    })
}
