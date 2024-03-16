use std::sync::Arc;

use frame_support::assert_ok;
use mp_felt::Felt252Wrapper;
use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, InvokeTransactionV1, TransactionSignature};

use super::constants::TOKEN_CONTRACT_CLASS_HASH;
use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::utils::build_get_balance_contract_call;

#[test]
fn given_call_contract_call_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(1);

        let origin = RuntimeOrigin::none();
        let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Deploy ERC20 Contract, as it is already declared in fixtures
        // Deploy ERC20 contract
        let constructor_calldata = Calldata(Arc::new(vec![
            sender_account.0.0, // Simple contract address
            StarkFelt::try_from("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8").unwrap(), // deploy_contract selector
            StarkFelt::try_from("0x0000000000000000000000000000000000000000000000000000000000000009").unwrap(), // Calldata len
            StarkFelt::try_from(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
            StarkFelt::ONE,                                             // Contract address salt
            StarkFelt::try_from("0x6").unwrap(),                     // Constructor_calldata_len
            StarkFelt::try_from("0xA").unwrap(),                     // Name
            StarkFelt::try_from("0x1").unwrap(),                     // Symbol
            StarkFelt::try_from("0x2").unwrap(),                     // Decimals
            StarkFelt::try_from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply low
            StarkFelt::try_from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply high
            sender_account.0.0,                                           // recipient
        ]));

        let deploy_transaction = InvokeTransactionV1 {
            sender_address: sender_account,
            signature: TransactionSignature(vec![]),
            nonce: Nonce(StarkFelt::ZERO),
            calldata: constructor_calldata,
            max_fee: Fee(u128::MAX),
        };

        assert_ok!(Starknet::invoke(origin, deploy_transaction.into()));

        let expected_erc20_address = ContractAddress(PatriciaKey(
            StarkFelt::try_from("00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap(),
        ));

        // Call balanceOf
        let call_args = build_get_balance_contract_call(sender_account.0 .0);
        pretty_assertions::assert_eq!(
            Starknet::call_contract(expected_erc20_address, call_args.0, call_args.1).unwrap(),
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        // Call symbol
        let symbol_selector = EntryPointSelector(
            StarkFelt::try_from("0x0216b05c387bab9ac31918a3e61672f4618601f3c598a2f3f2710f37053e1ea4").unwrap(),
        );
        let default_calldata = Calldata(Default::default());
        let res = Starknet::call_contract(expected_erc20_address, symbol_selector, default_calldata.clone()).unwrap();
        pretty_assertions::assert_eq!(res, vec![Felt252Wrapper::from_hex_be("0x01").unwrap()]);

        // Call name
        let name_selector = EntryPointSelector(
            StarkFelt::try_from("0x0361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60").unwrap(),
        );
        let res = Starknet::call_contract(expected_erc20_address, name_selector, default_calldata.clone()).unwrap();
        pretty_assertions::assert_eq!(res, vec![Felt252Wrapper::from_hex_be("0x0A").unwrap()]);

        // Call decimals
        let decimals_selector = EntryPointSelector(
            StarkFelt::try_from("0x004c4fb1ab068f6039d5780c68dd0fa2f8742cceb3426d19667778ca7f3518a9").unwrap(),
        );
        let res = Starknet::call_contract(expected_erc20_address, decimals_selector, default_calldata).unwrap();
        pretty_assertions::assert_eq!(res, vec![Felt252Wrapper::from_hex_be("0x02").unwrap()]);
    });
}
