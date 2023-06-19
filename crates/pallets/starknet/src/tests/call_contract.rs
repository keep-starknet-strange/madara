use frame_support::{assert_ok, bounded_vec};
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::InvokeTransaction;
use sp_core::ConstU32;
use sp_runtime::BoundedVec;

use super::constants::TOKEN_CONTRACT_CLASS_HASH;
use super::mock::*;

#[test]
fn given_call_contract_call_works() {
    new_test_ext().execute_with(|| {
        basic_test_setup(1);

        let origin = RuntimeOrigin::none();
        let sender_account = get_account_address(AccountType::NoValidate);

        // Deploy ERC20 Contract, as it is already declared in fixtures
        // Deploy ERC20 contract
        let constructor_calldata: BoundedVec<Felt252Wrapper, ConstU32<{ u32::MAX }>> = bounded_vec![
            sender_account, // Simple contract address
            Felt252Wrapper::from_hex_be("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8")
                .unwrap(), // deploy_contract selector
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000009")
                .unwrap(), // Calldata len
            Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
            Felt252Wrapper::ONE, // Contract address salt
            Felt252Wrapper::from_hex_be("0x6").unwrap(), // Constructor_calldata_len
            Felt252Wrapper::from_hex_be("0xA").unwrap(), // Name
            Felt252Wrapper::from_hex_be("0x1").unwrap(), // Symbol
            Felt252Wrapper::from_hex_be("0x2").unwrap(), // Decimals
            Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply low
            Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply high
            sender_account  // recipient
        ];

        let deploy_transaction = InvokeTransaction {
            version: 1,
            sender_address: sender_account,
            signature: bounded_vec!(),
            nonce: Felt252Wrapper::ZERO,
            calldata: constructor_calldata,
            max_fee: Felt252Wrapper::from(u128::MAX),
        };

        assert_ok!(Starknet::invoke(origin, deploy_transaction));

        let expected_erc20_address =
            Felt252Wrapper::from_hex_be("00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap();

        // Call balanceOf
        let balance_of_selector =
            Felt252Wrapper::from_hex_be("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap();
        let calldata = bounded_vec![
            sender_account // owner address
        ];
        let res = Starknet::call_contract(expected_erc20_address, balance_of_selector, calldata);
        assert_ok!(res.clone());
        pretty_assertions::assert_eq!(
            res.unwrap(),
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        // Call symbol
        let symbol_selector =
            Felt252Wrapper::from_hex_be("0x0216b05c387bab9ac31918a3e61672f4618601f3c598a2f3f2710f37053e1ea4").unwrap();
        let calldata = bounded_vec![];
        let res = Starknet::call_contract(expected_erc20_address, symbol_selector, calldata);
        assert_ok!(res.clone());
        pretty_assertions::assert_eq!(res.unwrap(), vec![Felt252Wrapper::from_hex_be("0x01").unwrap()]);

        // Call name
        let name_selector =
            Felt252Wrapper::from_hex_be("0x0361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60").unwrap();
        let calldata = bounded_vec![];
        let res = Starknet::call_contract(expected_erc20_address, name_selector, calldata);
        assert_ok!(res.clone());
        pretty_assertions::assert_eq!(res.unwrap(), vec![Felt252Wrapper::from_hex_be("0x0A").unwrap()]);

        // Call decimals
        let decimals_selector =
            Felt252Wrapper::from_hex_be("0x004c4fb1ab068f6039d5780c68dd0fa2f8742cceb3426d19667778ca7f3518a9").unwrap();
        let calldata = bounded_vec![];
        let res = Starknet::call_contract(expected_erc20_address, decimals_selector, calldata);
        assert_ok!(res.clone());
        pretty_assertions::assert_eq!(res.unwrap(), vec![Felt252Wrapper::from_hex_be("0x02").unwrap()]);
    });
}
