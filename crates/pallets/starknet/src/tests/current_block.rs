use frame_support::debug;
use mp_starknet::block::Header as StarknetHeader;
use mp_starknet::execution::types::Felt252Wrapper;
use sp_core::U256;

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
            block_number: U256::from(2),
            parent_block_hash: Felt252Wrapper::from_hex_be(
                "0x01243efd82a868d20c15c273d185467feb4addc129fb767353fa684e186d3f98",
            )
            .unwrap(),
            transaction_count: 1,
            // This expected value has been computed in the sequencer test (commitment on a tx hash 0 without
            // signature).
            transaction_commitment: Felt252Wrapper::from_hex_be(
                "0x039050b107da7374213fffb38becd5f2d76e51ffa0734bf5c7f8f0477a6f2c22",
            )
            .unwrap(),
            event_count: 2,
            event_commitment: Felt252Wrapper::from_hex_be(
                "0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02",
            )
            .unwrap(),
            sequencer_address: Felt252Wrapper::try_from(&SEQUENCER_ADDRESS).unwrap(),
            ..StarknetHeader::default()
        };

        pretty_assertions::assert_eq!(*current_block.header(), expected_current_block);
        pretty_assertions::assert_eq!(current_block.transactions_hashes().len(), 1);
        pretty_assertions::assert_eq!(
            current_block.transactions_hashes().get(0).unwrap(),
            &Felt252Wrapper::from_hex_be("0x1").unwrap()
        );
        debug(&current_block);
    });
}
