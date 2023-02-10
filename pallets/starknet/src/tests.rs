use crate::mock::*;

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
	new_test_ext().execute_with(|| {
		let deployer_account = 1;
		let _deployer_origin = RuntimeOrigin::signed(deployer_account);
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// TODO: Implement this
	});
}
