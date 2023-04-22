use core::str::FromStr;

use blockifier::execution::contract_class::ContractClass;
use frame_support::{assert_ok, bounded_vec};
use hexlit::hex;
use lazy_static::lazy_static;
use mp_starknet::execution::{
    CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper,
};
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{EventWrapper, Transaction};
use sp_core::H256;

use super::mock::*;
use crate::Event;

fn get_contract_class_wrapper(contract_content: &'static [u8]) -> ContractClassWrapper {
    let contract_class: ContractClass =
        serde_json::from_slice(contract_content).expect("File must contain the content of a compiled contract.");
    ContractClassWrapper::from(contract_class)
}

lazy_static! {
    static ref ERC20_CONTRACT_CLASS: ContractClassWrapper =
        get_contract_class_wrapper(include_bytes!("../../../../../resources/erc20/erc20.json"));
}
const ERC20_CLASS_HASH: [u8; 32] = hex!("01d1aacf8f874c4a865b974236419a46383a5161925626e9053202d8e87257e9");

#[test]
fn given_erc20_transfer_when_invoke_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(1);
        let origin = RuntimeOrigin::none();
        let (sender_account, _, _) = account_helper(TEST_ACCOUNT_SALT);
        // Declare ERC20 contract
        declare_erc20(origin.clone(), sender_account);
        // Deploy ERC20 contract
        deploy_erc20(origin.clone(), sender_account);
        // TODO: use dynamic values to craft invoke transaction
        // Transfer some token
        invoke_transfer_erc20(origin, sender_account);
        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000f").unwrap(),
                    H256::from_str("0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap(),
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                ),
                from_address: H256::from_str("0x0074c41dd9ba722396796cba415f8a742d671eb872371c96ce1ce6016fd0f2bb")
                    .unwrap()
                    .to_fixed_bytes(),
            })
            .into(),
        );
    })
}

/// Helper function to declare ERC20 contract.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn declare_erc20(origin: RuntimeOrigin, sender_account: ContractAddressWrapper) {
    let declare_transaction = Transaction {
        sender_address: sender_account,
        call_entrypoint: CallEntryPointWrapper::new(
            Some(ERC20_CLASS_HASH),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![],
            sender_account,
            sender_account,
        ),
        contract_class: Some(ERC20_CONTRACT_CLASS.clone()),
        ..Transaction::default()
    };
    assert_ok!(Starknet::declare(origin, declare_transaction));
}

/// Helper function to deploy ERC20 contract.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn deploy_erc20(origin: RuntimeOrigin, _sender_account: ContractAddressWrapper) {
    let deploy_transaction = transaction_from_json(
        include_str!("../../../../../resources/transactions/deploy_erc20.json"),
        include_bytes!("../../../../../resources/account/account.json"),
    )
    .unwrap();
    assert_ok!(Starknet::invoke(origin, deploy_transaction));
}

/// Helper function to mint some tokens.
/// # Arguments
/// * `origin` - The origin of the transaction.
/// * `sender_account` - The address of the sender account.
fn invoke_transfer_erc20(origin: RuntimeOrigin, _sender_account: ContractAddressWrapper) {
    let erc20_mint_tx_json: &str = include_str!("../../../../../resources/transactions/invoke_erc20_transfer.json");
    let erc20_mint_tx = transaction_from_json(erc20_mint_tx_json, &[]).expect("Failed to create Transaction from JSON");
    assert_ok!(Starknet::invoke(origin, erc20_mint_tx));
}
