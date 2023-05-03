use core::str::FromStr;

use frame_support::{assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::execution::types::{CallEntryPointWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::Transaction;
use sp_core::{ConstU32, H256, U256};
use sp_runtime::BoundedVec;

use super::mock::*;

#[test]
fn given_call_contract_call_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(1);

        let origin = RuntimeOrigin::none();
        // 0x0F == 15
        let sender_account =
            <[u8; 32]>::from_hex("000000000000000000000000000000000000000000000000000000000000000F").unwrap();

        // Deploy ERC20 Contract, as it is already declared in fixtures
        // Deploy ERC20 contract
        let constructor_calldata: BoundedVec<sp_core::U256, ConstU32<{ u32::MAX }>> = bounded_vec![
            U256::from(15), // Simple contract address
            U256::from_str("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8").unwrap(), // deploy_contract selector
            U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000009").unwrap(), // Calldata len
            U256::from_str(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
            U256::one(), // Contract address salt
            U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").unwrap(), // Constructor_calldata_len
            U256::from_str("0x000000000000000000000000000000000000000000000000000000000000000A").unwrap(), // Name
            U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), // Symbol
            U256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(), // Decimals
            U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply low
            U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply high
            U256::from_big_endian(&sender_account) // recipient
        ];
        let deploy_transaction = Transaction {
            version: 1,
            sender_address: sender_account,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(<[u8; 32]>::from_hex(TOKEN_CONTRACT_CLASS_HASH).unwrap()),
                EntryPointTypeWrapper::External,
                None,
                constructor_calldata,
                sender_account,
                sender_account,
            ),
            hash: H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap(),
            signature: bounded_vec!(),
            nonce: U256::one(),
            contract_class: None,
            contract_address_salt: None,
        };

        assert_ok!(Starknet::invoke(origin, deploy_transaction));

        let expected_erc20_address = <[u8; 32]>::from_hex("0348571287631347b50c7d2b7011b22349919ea14e7065a45b79632a6891c608").unwrap();

        // Call balanceOf
        let balance_of_selector =
            H256::from_str("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap();
        let calldata = bounded_vec![
            U256::from_big_endian(&sender_account) // owner address
        ];
        let res = Starknet::call_contract(expected_erc20_address, balance_of_selector, calldata);
        assert_ok!(res.clone());
		pretty_assertions::assert_eq!(res.unwrap(), vec![U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),U256::from_str("0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()]);


		// Call symbol
		let symbol_selector = H256::from_str("0x0216b05c387bab9ac31918a3e61672f4618601f3c598a2f3f2710f37053e1ea4").unwrap();
		let calldata = bounded_vec![];
		let res = Starknet::call_contract(expected_erc20_address, symbol_selector, calldata);
		assert_ok!(res.clone());
		pretty_assertions::assert_eq!(res.unwrap(), vec![U256::from_str("0x01").unwrap()]);

		// Call name
		let name_selector = H256::from_str("0x0361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60").unwrap();
		let calldata = bounded_vec![];
		let res = Starknet::call_contract(expected_erc20_address, name_selector, calldata);
		assert_ok!(res.clone());
		pretty_assertions::assert_eq!(res.unwrap(), vec![U256::from_str("0x0A").unwrap()]);

		// Call decimals
		let decimals_selector = H256::from_str("0x004c4fb1ab068f6039d5780c68dd0fa2f8742cceb3426d19667778ca7f3518a9").unwrap();
		let calldata = bounded_vec![];
		let res = Starknet::call_contract(expected_erc20_address, decimals_selector, calldata);
		assert_ok!(res.clone());
		pretty_assertions::assert_eq!(res.unwrap(), vec![U256::from_str("0x02").unwrap()]);
    });
}
