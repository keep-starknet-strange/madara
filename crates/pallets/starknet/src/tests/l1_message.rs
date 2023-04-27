use frame_support::{assert_err, bounded_vec};
use hex::FromHex;
use mp_starknet::execution::types::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::Transaction;

use super::mock::*;
use crate::tests::declare_tx::ERC20_CONTRACT_PATH;
use crate::Error;

#[test]
fn given_contract_l1_message_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let erc20_class = ContractClassWrapper::try_from(get_contract_class(ERC20_CONTRACT_PATH)).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = Transaction {
            sender_address: contract_address_bytes,
            contract_class: Some(erc20_class),
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                contract_address_bytes,
                contract_address_bytes,
            ),
            ..Transaction::default()
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::AccountNotDeployed);
    })
}
