use frame_support::debug;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::execution::types::Felt252Wrapper;

use super::mock::*;
use crate::SEQUENCER_ADDRESS;

#[test]
fn given_normal_conditions_when_current_block_then_returns_correct_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let current_block = Starknet::current_block();

        let expected_current_block = StarknetHeader {
            block_timestamp: 12,
            block_number: 2,
            parent_block_hash: Felt252Wrapper::from_hex_be(
                "0x05c25dd4c3fb1e97ccbc6dfc72f807e0800037ef39a336d143e0277beca886e5",
            )
            .unwrap(),
            transaction_count: 0,
            // This expected value has been computed in the sequencer test (commitment on a tx hash 0 without
            // signature).
            transaction_commitment: Felt252Wrapper::from_hex_be(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            event_count: 0,
            event_commitment: Felt252Wrapper::from_hex_be(
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            sequencer_address: Felt252Wrapper::try_from(&SEQUENCER_ADDRESS).unwrap(),
            ..StarknetHeader::default()
        };

        pretty_assertions::assert_eq!(*current_block.header(), expected_current_block);
        debug(&current_block);
    });
}
