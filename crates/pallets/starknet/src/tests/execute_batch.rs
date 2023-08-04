use core::str::from_utf8;

use blockifier::execution::contract_class::{ContractClass, ContractClassV1};
use frame_support::{assert_err, assert_ok, bounded_vec};
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction};

use super::mock::default_mock::*;
use super::mock::*;
use crate::Error;

const HELLO_SN_CLASS_HASH: &str = "0x00df4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87";
const HELLO_SN_SALT: &str = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
const HELLO_SN_GET_BALANCE_SELECTOR: &str = "0x39e11d48192e4333233c7eb19d10ad67c362bb28580c604d67884c85da39695";

// Troubleshooting: RUST_LOG=runtime::starknet=error RUST_BACKTRACE=1 cargo test --package
// pallet-starknet -- --nocapture

fn build_declare_transaction(sender_address: ContractAddressWrapper, nonce: Felt252Wrapper) -> DeclareTransaction {
    let contract_class_bytes = include_bytes!("../../../../../cairo-contracts/build/cairo_1/HelloStarknet.casm.json");
    let contract_class = ContractClassV1::try_from_json_string(from_utf8(contract_class_bytes).unwrap()).unwrap();

    let class_hash = Felt252Wrapper::from_hex_be(HELLO_SN_CLASS_HASH).unwrap();

    DeclareTransaction {
        version: 2_u8,
        sender_address,
        nonce,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        contract_class: ContractClass::V1(contract_class),
        compiled_class_hash: None,
        class_hash,
        is_query: false,
    }
}

fn from_declare_transaction(declare_tx: DeclareTransaction) -> Transaction {
    declare_tx.from_declare(Starknet::chain_id())
}

fn build_deploy_account_transaction(nonce: Felt252Wrapper) -> (DeployAccountTransaction, Felt252Wrapper) {
    let class_hash = Felt252Wrapper::from_hex_be(HELLO_SN_CLASS_HASH).unwrap();
    let salt = Felt252Wrapper::from_hex_be(HELLO_SN_SALT).unwrap();
    let address = calculate_contract_address(salt, class_hash.into(), bounded_vec!()).unwrap();

    let deploy_tx = DeployAccountTransaction {
        account_class_hash: class_hash,
        calldata: bounded_vec![],
        salt: Felt252Wrapper::from_hex_be(HELLO_SN_SALT).unwrap(),
        version: 1,
        nonce,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        is_query: false,
    };

    (deploy_tx, address.0.0.into())
}

fn from_deploy_transaction(deploy_tx: DeployAccountTransaction) -> Transaction {
    deploy_tx.from_deploy(Starknet::chain_id()).unwrap()
}

fn build_invoke_transaction(
    sender_address: ContractAddressWrapper,
    contract_address: Felt252Wrapper,
    nonce: Felt252Wrapper,
) -> InvokeTransaction {
    InvokeTransaction {
        version: 1_u8,
        sender_address,
        calldata: bounded_vec![
            contract_address,
            Felt252Wrapper::from_hex_be(HELLO_SN_GET_BALANCE_SELECTOR).unwrap(),
            Felt252Wrapper::ZERO, // Calldata len (get_balance has 0 arguments)
        ],
        nonce,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        is_query: false,
    }
}

fn from_invoke_transaction(invoke_tx: InvokeTransaction) -> Transaction {
    invoke_tx.from_invoke(Starknet::chain_id())
}

#[test]
fn execute_batch_with_skip_flags_succeeded() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let declare_tx = build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        let tx1 = from_declare_transaction(declare_tx);

        let (deploy_tx, contract_address) = build_deploy_account_transaction(Felt252Wrapper::ZERO);
        let tx2 = from_deploy_transaction(deploy_tx);

        let invoke_tx = build_invoke_transaction(sender_account, contract_address, Felt252Wrapper::from(1u128));
        let tx3 = from_invoke_transaction(invoke_tx);

        // HelloStarknet is not an account contract (does not implement `AccountContractImpl` trait)
        // But here we just want to test that:
        //  1) execute_batch handles declare, account_deploy, and invoke transactions
        //  2) execute_batch respects changes made by the previous txs in the batch
        //  3) skip flags work as expected
        // We need to disable validation with skip_validation` flag and it will not complain about missing
        // entrypoint Also deployed contract has to have non-empty balance, we workaround that by
        // setting `skip_fee_charge` flag
        match Starknet::execute_batch(vec![tx1, tx2, tx3], true, true) {
            Ok(mut results) => {
                assert!(results[0].execute_call_info.is_none());
                assert!(results[1].execute_call_info.is_some());
                assert!(results[2].execute_call_info.is_some());

                // Account.__execute__() -> Hello.get_balance()
                let balance = results[2].execute_call_info.as_ref().unwrap().inner_calls[0].execution.retdata.0[0];
                assert_eq!(Felt252Wrapper::from(balance), Felt252Wrapper::ZERO);

                for exec_info in results.drain(..) {
                    assert!(exec_info.validate_call_info.is_none());
                    assert!(exec_info.fee_transfer_call_info.is_none());
                }
            }
            Err(err) => panic!("{:?}", err),
        };
    });
}

#[test]
fn execute_batch_declare_for_declared_class_failed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let declare_tx = build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx.clone()));

        let mut tx = from_declare_transaction(declare_tx);
        tx.nonce = Felt252Wrapper::from(1u128);
        assert_err!(Starknet::execute_batch(vec![tx], false, false), Error::<MockRuntime>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn execute_batch_rollback_succeeded() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let declare_tx = build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        let tx1 = from_declare_transaction(declare_tx.clone());

        let (deploy_tx, _) = build_deploy_account_transaction(Felt252Wrapper::ZERO);
        let tx2 = from_deploy_transaction(deploy_tx);

        assert_err!(
            Starknet::execute_batch(vec![tx1, tx2], false, false),
            Error::<MockRuntime>::TransactionExecutionFailed
        );

        // Batch updates must be discarded at this point
        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx));
    });
}
