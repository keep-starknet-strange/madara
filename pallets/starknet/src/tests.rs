use frame_support::{assert_ok, BoundedVec};

use crate::{mock::*, Event};

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
	new_test_ext().execute_with(|| {
		let deployer_account = 1;
		let deployer_origin = RuntimeOrigin::signed(deployer_account);
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let sierra_code = BoundedVec::truncate_from(vec![1, 2, 3]);
		// Dispatch a signed extrinsic.
		assert_ok!(Starknet::ping(deployer_origin));
	});
}
