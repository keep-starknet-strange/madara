use frame_support::{assert_ok, BoundedVec};

use crate::mock::*;
use crate::Event;

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
    new_test_ext().execute_with(|| {
        let deployer_account = 1;
        let deployer_origin = RuntimeOrigin::signed(deployer_account);
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        let sierra_code = BoundedVec::truncate_from(vec![0; 32]);
        // Dispatch a signed extrinsic.
        assert_ok!(Cairo::deploy_sierra_program(deployer_origin, sierra_code.clone()));

        // Get sierra program id
        let sierra_program_id = Cairo::gen_sierra_program_id(&deployer_account, &sierra_code).unwrap();

        // Read pallet storage and assert an expected result.
        let sierra_program = Cairo::sierra_programs(sierra_program_id).unwrap();
        assert_eq!(sierra_program.code, sierra_code);
        // Assert that the correct event was deposited
        System::assert_last_event(Event::SierraProgramDeployed { deployer_account, id: sierra_program_id }.into());
    });
}
