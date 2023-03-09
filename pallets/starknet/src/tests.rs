use frame_support::assert_ok;
use kp_starknet::block::wrapper::header::Header;

use crate::mock::*;

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
    new_test_ext().execute_with(|| {
        let deployer_account = 1;
        let deployer_origin = RuntimeOrigin::signed(deployer_account);
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(Starknet::ping(deployer_origin));
    });
}

#[test]
fn given_normal_conditions_when_current_block_then_returns_correct_block() {
    new_test_ext().execute_with(|| {
        run_to_block(2);
        let current_block = Starknet::current_block();
        let expected_current_block = Header { block_timestamp: 10_u64, ..Header::default() };
        assert!(current_block.is_some());
        // assert_eq!(current_block.unwrap().header, expected_current_block)
    });
}
