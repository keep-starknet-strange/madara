use mp_felt::Felt252Wrapper;
use mp_transactions::execution::Execute;
use mp_transactions::{DeployAccountTransaction, HandleL1MessageTransaction, UserOrL1HandlerTransaction};
use starknet_api::api_core::{ContractAddress, Nonce};
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;
use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::execution_config::RuntimeExecutionConfigBuilder;
use crate::tests::utils::get_contract_class;
use crate::tests::{constants, get_declare_dummy, get_invoke_dummy, set_infinite_tokens};
use crate::types::CasmClassHash;
use crate::Config;

#[test]
fn re_execute_tx_ok() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let invoke_sender_address: ContractAddress =
            Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap().into();
        let chain_id = Starknet::chain_id();

        // Deploy

        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt =
            Felt252Wrapper::from_hex_be("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let deploy_tx = DeployAccountTransaction {
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            contract_address_salt: salt,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
            offset_version: false,
        };

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);

        // Declare

        let declare_tx =
            get_declare_dummy(chain_id, Felt252Wrapper::ZERO, AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        let erc20_class_hash: CasmClassHash =
            Felt252Wrapper::from_hex_be("0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4")
                .unwrap()
                .into();
        let erc20_class = get_contract_class("ERC20.json", 0);

        let contract_address =
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let from_address = Felt252Wrapper::from_hex_be("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();

        // Handle l1 message

        let handle_l1_tx = HandleL1MessageTransaction {
            nonce: 1,
            contract_address,
            entry_point_selector: Felt252Wrapper::from_hex_be(
                "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269", // test_l1_handler_store_under_caller_address
            )
            .unwrap(),
            calldata: vec![
                from_address,
                Felt252Wrapper::from_hex_be("0x1").unwrap(), // value
            ],
        };

        let txs = vec![
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Invoke(
                get_invoke_dummy(Felt252Wrapper::ZERO).into(),
            )),
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Invoke(
                get_invoke_dummy(Felt252Wrapper::ONE).into(),
            )),
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Declare(declare_tx, erc20_class)),
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::DeployAccount(deploy_tx)),
            UserOrL1HandlerTransaction::L1Handler(handle_l1_tx, Fee(10)),
        ];

        // Call the function we want to test
        let res = Starknet::re_execute_transactions(txs.clone()).unwrap().unwrap();

        // Storage changes have been reverted
        assert_eq!(Starknet::nonce(invoke_sender_address), Nonce(Felt252Wrapper::ZERO.into()));
        assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), None);
        // All txs are there
        assert_eq!(res.len(), 5);

        // Now let's check the TransactionInfos returned
        let first_invoke_tx_info = match txs.get(0).unwrap() {
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Invoke(invoke_tx)) => invoke_tx
                .into_executable::<<MockRuntime as Config>::SystemHash>(chain_id, false)
                .execute(
                    &mut BlockifierStateAdapter::<MockRuntime>::default(),
                    &Starknet::get_block_context(),
                    &RuntimeExecutionConfigBuilder::new::<MockRuntime>().build(),
                )
                .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(res[0], first_invoke_tx_info);
        let second_invoke_tx_info = match txs.get(1).unwrap() {
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Invoke(invoke_tx)) => invoke_tx
                .into_executable::<<MockRuntime as Config>::SystemHash>(chain_id, false)
                .execute(
                    &mut BlockifierStateAdapter::<MockRuntime>::default(),
                    &Starknet::get_block_context(),
                    &RuntimeExecutionConfigBuilder::new::<MockRuntime>().build(),
                )
                .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(res[1], second_invoke_tx_info);
        let declare_tx_info = match txs.get(2).unwrap() {
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::Declare(declare_tx, cc)) => declare_tx
                .try_into_executable::<<MockRuntime as Config>::SystemHash>(chain_id, cc.clone(), false)
                .unwrap()
                .execute(
                    &mut BlockifierStateAdapter::<MockRuntime>::default(),
                    &Starknet::get_block_context(),
                    &RuntimeExecutionConfigBuilder::new::<MockRuntime>().build(),
                )
                .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(res[2], declare_tx_info);
        let deploy_account_tx_info = match txs.get(3).unwrap() {
            UserOrL1HandlerTransaction::User(mp_transactions::UserTransaction::DeployAccount(deploy_account_tx)) => {
                deploy_account_tx
                    .into_executable::<<MockRuntime as Config>::SystemHash>(chain_id, false)
                    .execute(
                        &mut BlockifierStateAdapter::<MockRuntime>::default(),
                        &Starknet::get_block_context(),
                        &RuntimeExecutionConfigBuilder::new::<MockRuntime>().build(),
                    )
                    .unwrap()
            }
            _ => unreachable!(),
        };
        assert_eq!(res[3], deploy_account_tx_info);
        let handle_l1_message_tx_info = match txs.get(4).unwrap() {
            UserOrL1HandlerTransaction::L1Handler(l1_tx, fee) => l1_tx
                .into_executable::<<MockRuntime as Config>::SystemHash>(chain_id, *fee, false)
                .execute(
                    &mut BlockifierStateAdapter::<MockRuntime>::default(),
                    &Starknet::get_block_context(),
                    &RuntimeExecutionConfigBuilder::new::<MockRuntime>().build(),
                )
                .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(res[4], handle_l1_message_tx_info);
    });
}
