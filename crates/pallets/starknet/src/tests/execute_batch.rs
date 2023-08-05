use core::str::from_utf8;

use blockifier::execution::contract_class::{ContractClass, ContractClassV1};
use frame_support::{assert_err, assert_ok, bounded_vec};
use mp_starknet::crypto::commitment::calculate_transaction_hash_common;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::execution::call_entrypoint_wrapper::{CallEntryPointWrapper, MaxCalldataSize};
use mp_starknet::execution::entrypoint_wrapper::EntryPointTypeWrapper;
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::{
    DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction, TxType,
};
use mp_starknet::transaction::utils::calculate_transaction_version_from_u8;
use sp_runtime::BoundedVec;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::get_contract_class;
use crate::Error;

// Generate: starkli class-hash ./cairo-contracts/build/cairo_1/HelloStarknet.casm.json
const HELLO_SN_CLASS_HASH: &str = "0x00df4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87";
const HELLO_SN_SALT: &str = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
const HELLO_SN_GET_BALANCE_SELECTOR: &str = "0x39e11d48192e4333233c7eb19d10ad67c362bb28580c604d67884c85da39695";

// Generate: starkli class-hash ./cairo-contracts/build/l1_handler.json
const L1_HANDLER_CLASS_HASH: &str = "0x065181b6a0ed210a60bcb64b09849ae7d14c9edaedccb8b50159eb062f7bdbe3";
const L1_HANDLER_SELECTOR: &str = "0x1310e2c127c3b511c5ac0fd7949d544bb4d75b8bc83aaeb357e712ecf582771";

// NoValidateAccount (Cairo 0)
const DEPLOY_CONTRACT_SELECTOR: &str = "0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8";

// Troubleshooting:
// RUST_LOG=runtime=error RUST_BACKTRACE=1 cargo test --package pallet-starknet -- --nocapture

fn hello_sn_build_declare_transaction(
    sender_address: ContractAddressWrapper,
    nonce: Felt252Wrapper,
) -> DeclareTransaction {
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

fn l1_handler_build_declare_transaction(
    sender_address: ContractAddressWrapper,
    nonce: Felt252Wrapper,
) -> DeclareTransaction {
    let contract_class = get_contract_class("l1_handler.json", 0);
    let class_hash = Felt252Wrapper::from_hex_be(L1_HANDLER_CLASS_HASH).unwrap();

    DeclareTransaction {
        sender_address,
        version: 1,
        class_hash,
        compiled_class_hash: None,
        contract_class,
        nonce,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        is_query: false,
    }
}

fn from_declare_transaction(declare_tx: DeclareTransaction) -> Transaction {
    declare_tx.from_declare(Starknet::chain_id())
}

fn hello_sn_build_deploy_account_transaction(nonce: Felt252Wrapper) -> (DeployAccountTransaction, Felt252Wrapper) {
    let class_hash = Felt252Wrapper::from_hex_be(HELLO_SN_CLASS_HASH).unwrap();
    let salt = Felt252Wrapper::from_hex_be(HELLO_SN_SALT).unwrap();
    let address = calculate_contract_address(salt, class_hash, bounded_vec!()).unwrap();

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

fn hello_sn_build_invoke_transaction(
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

fn l1_handler_build_contract_deploy_transaction(
    sender_address: ContractAddressWrapper,
    nonce: Felt252Wrapper,
) -> (InvokeTransaction, Felt252Wrapper) {
    let class_hash = Felt252Wrapper::from_hex_be(L1_HANDLER_CLASS_HASH).unwrap();
    let salt = Felt252Wrapper::ONE;
    let address = calculate_contract_address(salt, class_hash, bounded_vec!()).unwrap();

    let invoke_tx = InvokeTransaction {
        version: 1_u8,
        sender_address,
        calldata: bounded_vec![
            sender_address,
            Felt252Wrapper::from_hex_be(DEPLOY_CONTRACT_SELECTOR).unwrap(),
            Felt252Wrapper::from(3u128), // Calldata len
            class_hash,
            salt,
            Felt252Wrapper::ZERO, // Constructor calldata len (no explicit constructor declared)
        ],
        nonce,
        max_fee: Felt252Wrapper::from(u128::MAX),
        signature: bounded_vec!(),
        is_query: false,
    };

    (invoke_tx, address.0.0.into())
}

fn from_invoke_transaction(invoke_tx: InvokeTransaction) -> Transaction {
    invoke_tx.from_invoke(Starknet::chain_id())
}

fn l1_handler_build_l1_handler_transaction(contract_address: Felt252Wrapper, nonce: Felt252Wrapper) -> Transaction {
    let sender_address: ContractAddressWrapper = Felt252Wrapper::ONE;
    let max_fee = Felt252Wrapper::from(u128::MAX);

    let calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize> = bounded_vec![
        Felt252Wrapper::ONE, // calldata len
        Felt252Wrapper::ONE, // argument (has to be 1 to pass assertion)
    ];

    let hash = calculate_transaction_hash_common::<PedersenHasher>(
        sender_address,
        calldata.as_slice(),
        max_fee,
        nonce,
        calculate_transaction_version_from_u8(false, 1_u8),
        b"invoke",
        Starknet::chain_id(),
        None,
    );

    Transaction {
        tx_type: TxType::L1Handler,
        version: 1_u8,
        hash,
        signature: bounded_vec!(),
        sender_address,
        nonce,
        call_entrypoint: CallEntryPointWrapper::new(
            None,
            EntryPointTypeWrapper::External,
            Some(Felt252Wrapper::from_hex_be(L1_HANDLER_SELECTOR).unwrap()),
            calldata,
            contract_address,
            sender_address,
            Felt252Wrapper::ZERO,
            None,
        ),
        contract_class: None,
        contract_address_salt: None,
        max_fee,
        is_query: false,
    }
}

#[test]
fn execute_batch_with_skip_flags_succeed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let declare_tx = hello_sn_build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        let tx1 = from_declare_transaction(declare_tx);

        let (deploy_tx, contract_address) = hello_sn_build_deploy_account_transaction(Felt252Wrapper::ZERO);
        let tx2 = from_deploy_transaction(deploy_tx);

        let invoke_tx =
            hello_sn_build_invoke_transaction(sender_account, contract_address, Felt252Wrapper::from(1u128));
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
        let declare_tx = hello_sn_build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx.clone()));

        let mut tx = from_declare_transaction(declare_tx);
        tx.nonce = Felt252Wrapper::from(1u128);
        assert_err!(Starknet::execute_batch(vec![tx], false, false), Error::<MockRuntime>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn execute_batch_double_declare_failed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let declare_tx = hello_sn_build_declare_transaction(sender_account, Felt252Wrapper::ZERO);

        let tx1 = from_declare_transaction(declare_tx.clone());
        let mut tx2 = from_declare_transaction(declare_tx);
        tx2.nonce = Felt252Wrapper::from(1u128);
        assert_err!(
            Starknet::execute_batch(vec![tx1, tx2], false, false),
            Error::<MockRuntime>::ClassHashAlreadyDeclared
        );
    });
}

#[test]
fn execute_batch_rollback_succeed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let declare_tx = hello_sn_build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        let tx1 = from_declare_transaction(declare_tx.clone());

        let (deploy_tx, _) = hello_sn_build_deploy_account_transaction(Felt252Wrapper::ZERO);
        let tx2 = from_deploy_transaction(deploy_tx);

        assert_err!(
            Starknet::execute_batch(vec![tx1, tx2], false, false),
            Error::<MockRuntime>::TransactionExecutionFailed
        );

        // Batch updates must be discarded at this point
        assert_ok!(Starknet::declare(RuntimeOrigin::none(), declare_tx));
    });
}

#[test]
fn execute_batch_l1_handler_call_succeed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let sender_account = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let declare_tx = l1_handler_build_declare_transaction(sender_account, Felt252Wrapper::ZERO);
        let tx1 = from_declare_transaction(declare_tx);

        let (invoke_tx, contract_address) =
            l1_handler_build_contract_deploy_transaction(sender_account, Felt252Wrapper::ONE);
        let tx2 = from_invoke_transaction(invoke_tx);

        let tx3 = l1_handler_build_l1_handler_transaction(contract_address, Felt252Wrapper::ZERO);

        match Starknet::execute_batch(vec![tx1, tx2, tx3], false, false) {
            Ok(exec_info) => {
                assert!(exec_info[0].execute_call_info.is_none());
                assert!(exec_info[0].validate_call_info.is_some());
                assert!(exec_info[0].fee_transfer_call_info.is_some());

                assert!(exec_info[1].execute_call_info.is_some());
                assert!(exec_info[1].validate_call_info.is_some());
                assert!(exec_info[1].fee_transfer_call_info.is_some());

                assert!(exec_info[2].execute_call_info.is_some());
                assert!(exec_info[2].validate_call_info.is_none());
                assert!(exec_info[2].fee_transfer_call_info.is_none());
            }
            Err(err) => panic!("{:?}", err),
        };
    });
}
