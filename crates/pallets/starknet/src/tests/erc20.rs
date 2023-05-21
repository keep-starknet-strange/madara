use core::str::FromStr;

use frame_support::{assert_ok, bounded_vec};
use lazy_static::lazy_static;
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::{EventWrapper, InvokeTransaction};
use sp_core::{H256, U256};

use super::mock::*;
use super::utils::get_contract_class_wrapper;
use crate::tests::constants::TOKEN_CONTRACT_CLASS_HASH;
use crate::Event;

lazy_static! {
    static ref ERC20_CONTRACT_CLASS: ContractClassWrapper = get_contract_class_wrapper("erc20/erc20.json");
}

#[test]
fn given_erc20_transfer_when_invoke_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(1);
        let origin = RuntimeOrigin::none();
        let sender_account = get_account_address(AccountType::NoValidate);
        // ERC20 is already declared for the fees.
        // Deploy ERC20 contract
        let deploy_transaction = InvokeTransaction {
            version: 1,
            sender_address: sender_account,
            calldata: bounded_vec![
                    sender_account, // Simple contract address
                    Felt252Wrapper(U256::from_str("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8").unwrap()), // deploy_contract selector
                    Felt252Wrapper(U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000009").unwrap()), // Calldata len
                    Felt252Wrapper(U256::from_str(TOKEN_CONTRACT_CLASS_HASH).unwrap()), // Class hash
                    Felt252Wrapper::one(), // Contract address salt
                    Felt252Wrapper(U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").unwrap()), // Constructor_calldata_len
                    Felt252Wrapper(U256::from_str("0x000000000000000000000000000000000000000000000000000000000000000A").unwrap()), // Name
                    Felt252Wrapper(U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()), // Symbol
                    Felt252Wrapper(U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap()), // Decimals
                    Felt252Wrapper(U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()), // Initial supply low
                    Felt252Wrapper(U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()), // Initial supply high
                    sender_account // recipient
                ],
            signature: bounded_vec!(),
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
        };
        let expected_erc20_address = H256::from_str("00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap().into();

        assert_ok!(Starknet::invoke(origin.clone(), deploy_transaction));
        let events = System::events();
        // Check transaction event (deployment)
        pretty_assertions::assert_eq!(
            Event::<MockRuntime>::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x026b160f10156dea0639bec90696772c640b9706a47f5b8c52ea1abe5858b34d").unwrap().into()
                ],
                data: bounded_vec!(
                    expected_erc20_address, // Contract address
                    Felt252Wrapper::zero(), // Deployer (always 0 with this account contract)
                    H256::from_str(TOKEN_CONTRACT_CLASS_HASH).unwrap().into(), // Class hash
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").unwrap().into(), // Constructor calldata len
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000a").unwrap().into(), // Name
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(), // Symbol
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap().into(), // Decimals
                    H256::from_str("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff").unwrap().into(), // Initial supply low
                    H256::from_str("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff").unwrap().into(), // Initial supply high
                    H256::from_str("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap().into(), // Recipient
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(), // Salt
                ),
                from_address: sender_account,
            }),
            events[events.len() - 2].event.clone().try_into().unwrap(),
        );
        let expected_fee_transfer_event = Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap().into()
                ],
                data: bounded_vec!(
                    sender_account, // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap().into(), // Sequencer address
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000002b660").unwrap().into(), // Amount low
                    Felt252Wrapper::zero(), // Amount high
                ),
                from_address:Starknet::fee_token_address(),
            });
        // Check fee transfer event
        pretty_assertions::assert_eq!(expected_fee_transfer_event, events.last().unwrap().event.clone().try_into().unwrap());
        // TODO: use dynamic values to craft invoke transaction
        // Transfer some token
        let transfer_transaction = InvokeTransaction {
            version: 1,
            sender_address: sender_account,
            calldata: bounded_vec![
                    expected_erc20_address, // Token address
                    Felt252Wrapper(U256::from_str("0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e").unwrap()), // transfer selector
                    Felt252Wrapper(U256::from(3)), // Calldata len
                    Felt252Wrapper(U256::from(16)), // recipient
                    Felt252Wrapper(U256::from(15)), // initial supply low
                    Felt252Wrapper::zero(), // initial supply high
                ],
            signature: bounded_vec!(),
            nonce: U256::one(),
            max_fee: U256::from(u128::MAX),
        };
        // Also asserts that the deployment has been saved.
        assert_ok!(Starknet::invoke(origin, transfer_transaction));
        pretty_assertions::assert_eq!(Starknet::storage((expected_erc20_address, Into::<Felt252Wrapper>::into(H256::from_str("03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb403f").unwrap()))),Felt252Wrapper(U256::from_str("ffffffffffffffffffffffffffffff0").unwrap()));
        pretty_assertions::assert_eq!(Starknet::storage((expected_erc20_address, Into::<Felt252Wrapper>::into(H256::from_str("03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb4040").unwrap()))),Felt252Wrapper(U256::from_str("fffffffffffffffffffffffffffffff").unwrap()));

        pretty_assertions::assert_eq!(Starknet::storage((expected_erc20_address, Into::<Felt252Wrapper>::into(H256::from_str("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73e").unwrap()))),Felt252Wrapper(U256::from(15)));
        pretty_assertions::assert_eq!(Starknet::storage((expected_erc20_address, Into::<Felt252Wrapper>::into(H256::from_str("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73f").unwrap()))),Felt252Wrapper::zero());

        let events = System::events();
        // Check regular event.
        let expected_event = Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap().into()
                ],
                data: bounded_vec!(
                    H256::from_str("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap().into(), // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000010").unwrap().into(), // To
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000F").unwrap().into(), // Amount low
                    Felt252Wrapper::zero(), // Amount high
                ),
                from_address: H256::from_str("0x00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff")
                    .unwrap()
                    .into(),
            });
        pretty_assertions::assert_eq!(expected_event, events[events.len() - 2].event.clone().try_into().unwrap());
        // Check fee transfer.
        let expected_fee_transfer_event = Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap().into()
                ],
                data: bounded_vec!(
                    sender_account, // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap().into(), // Sequencer address
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000001e618").unwrap().into(), // Amount low
                    Felt252Wrapper::zero(), // Amount high
                ),
                from_address:Starknet::fee_token_address(),
            });
        pretty_assertions::assert_eq!(expected_fee_transfer_event, events.last().unwrap().event.clone().try_into().unwrap());

    })
}
