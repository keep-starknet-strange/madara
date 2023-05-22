use frame_support::assert_err;
use hex::FromHex;
use mp_starknet::execution::types::ContractClassWrapper;
use mp_starknet::transaction::types::DeclareTransaction;

use super::mock::*;
use super::utils::get_contract_class;
use crate::Error;

#[test]
fn given_contract_l1_message_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap().into();

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();

        let transaction = DeclareTransaction {
            sender_address: contract_address_bytes,
            contract_class: erc20_class,
            ..DeclareTransaction::default()
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}
