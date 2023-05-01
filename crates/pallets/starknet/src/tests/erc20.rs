use core::str::FromStr;

use blockifier::execution::contract_class::ContractClass;
use frame_support::{assert_ok, bounded_vec};
use hex::FromHex;
use lazy_static::lazy_static;
use mp_starknet::execution::types::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::{EventWrapper, Transaction};
use sp_core::{H256, U256};

use super::mock::*;
use crate::Event;

fn get_contract_class_wrapper(contract_content: &'static [u8]) -> ContractClassWrapper {
    let contract_class: ContractClass =
        serde_json::from_slice(contract_content).expect("File must contain the content of a compiled contract.");
    ContractClassWrapper::try_from(contract_class).unwrap()
}

lazy_static! {
    static ref ERC20_CONTRACT_CLASS: ContractClassWrapper =
        get_contract_class_wrapper(include_bytes!("../../../../../resources/erc20/erc20.json"));
}

#[test]
fn given_erc20_transfer_when_invoke_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(1);
        let origin = RuntimeOrigin::none();
        let sender_account = <[u8; 32]>::from_hex("000000000000000000000000000000000000000000000000000000000000000F").unwrap();
        // ERC20 is already declared for the fees.
        // Deploy ERC20 contract
        let deploy_transaction = Transaction {
            version: 1,
            sender_address: sender_account,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(<[u8;32]>::from_hex(TOKEN_CONTRACT_CLASS_HASH).unwrap()),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
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
                ],
                sender_account,
                sender_account,
            ),
            hash: H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap(),
            signature: bounded_vec!(),
            nonce: U256::one(),
            contract_class: None,
            contract_address_salt: None,
        };
        let expected_erc20_address = "0348571287631347b50c7d2b7011b22349919ea14e7065a45b79632a6891c608";

        assert_ok!(Starknet::invoke(origin.clone(), deploy_transaction));

         System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x026b160f10156dea0639bec90696772c640b9706a47f5b8c52ea1abe5858b34d").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str(expected_erc20_address).unwrap(), // Contract address
                    H256::zero(), // Deployer (always 0 with this account contract)
                    H256::from_str(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").unwrap(), // Constructor calldata len
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000a").unwrap(), // Name
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), // Symbol
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(), // Decimals
                    H256::from_str("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff").unwrap(), // Initial supply low
                    H256::from_str("0x000000000000000000000000000000000fffffffffffffffffffffffffffffff").unwrap(), // Initial supply high
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000f").unwrap(), // Recipient
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), // Salt
                ),
                from_address: sender_account,
            })
            .into(),
        );

        // TODO: use dynamic values to craft invoke transaction
        // Transfer some token
        let transfer_transaction = Transaction {
            version: 1,
            sender_address: sender_account,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(<[u8;32]>::from_hex("06232eeb9ecb5de85fc927599f144913bfee6ac413f2482668c9f03ce4d07922").unwrap()),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![
                    U256::from_str(expected_erc20_address).unwrap(), // Token address
                    U256::from_str("0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e").unwrap(), // transfer selector
                    U256::from(3), // Calldata len
                    U256::from(16), // recipient
                    U256::from(15), // initial supply low
                    U256::zero(), // initial supply high
                ],
                sender_account,
                sender_account,
            ),
            hash: H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d213").unwrap(),
            signature: bounded_vec!(),
            nonce: U256::one(),
            contract_class: None,
            contract_address_salt: None,
        };
        // Also asserts that the deployment has been saved.
        assert_ok!(Starknet::invoke(origin, transfer_transaction));
        pretty_assertions::assert_eq!(Starknet::storage((<[u8; 32]>::from_hex(expected_erc20_address).unwrap(),H256::from_str("078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap())),U256::from_str("ffffffffffffffffffffffffffffff0").unwrap());
        pretty_assertions::assert_eq!(Starknet::storage((<[u8; 32]>::from_hex(expected_erc20_address).unwrap(),H256::from_str("078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap())),U256::from_str("fffffffffffffffffffffffffffffff").unwrap());

        pretty_assertions::assert_eq!(Starknet::storage((<[u8; 32]>::from_hex(expected_erc20_address).unwrap(),H256::from_str("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73e").unwrap())),U256::from(15));
        pretty_assertions::assert_eq!(Starknet::storage((<[u8; 32]>::from_hex(expected_erc20_address).unwrap(),H256::from_str("0x011cb0dc747a73020cbd50eac7460edfaa7d67b0e05823b882b05c3f33b1c73f").unwrap())),U256::zero());


        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000F").unwrap(), // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000010").unwrap(), // To
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000000F").unwrap(), // Amount low
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(), // Amount high
                ),
                from_address: H256::from_str("0x0348571287631347b50c7d2b7011b22349919ea14e7065a45b79632a6891c608")
                    .unwrap()
                    .to_fixed_bytes(),
            })
            .into(),
        );
    })
}
