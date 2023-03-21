use frame_support::BoundedVec;
use kp_starknet::execution::{CallEntryPointWrapper, EntryPointTypeWrapper};
use kp_starknet::transaction::types::Transaction;
use sp_core::{H256, U256};

use crate::pallet::alloc::borrow::ToOwned;
use crate::pallet::alloc::format;
use crate::pallet::alloc::string::String;
use crate::pallet::alloc::vec::Vec;
use crate::types::{ContractAddress, Message};

impl Message {
    pub fn decode_hex(s: &str) -> [u8; 32] {
        let s = s.trim_start_matches("0x");
        let s = if s.len() % 2 != 0 { format!("0{:}", s) } else { s.to_owned() };

        let decoded =
            (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect::<Vec<u8>>();
        core::array::from_fn(|i| if i < decoded.len() { decoded[i] } else { 0 })
    }
    pub fn to_transaction(&self) -> Transaction {
        let sender_address = Self::decode_hex(&self.topics[2]);
        let selector = H256::from_slice(&Self::decode_hex(&self.topics[3]));
        let char_vec = self.data.trim_start_matches("0x").chars().collect::<Vec<char>>();

        let data_map = char_vec.chunks(32).map(|chunk| chunk.iter().collect::<String>());
        let nonce = U256::from_str_radix(&data_map.clone().last().unwrap(), 16).unwrap();
        let calldata = BoundedVec::try_from(
            data_map
                .take(self.data.len() - 2)
                .map(|val| H256::from_slice(&Self::decode_hex(&val)))
                .collect::<Vec<H256>>(),
        )
        .unwrap();
        let call_entrypoint = CallEntryPointWrapper {
            class_hash: None,
            entrypoint_type: EntryPointTypeWrapper::L1Handler,
            entrypoint_selector: Some(selector),
            calldata,
            storage_address: sender_address,
            caller_address: ContractAddress::default(),
        };
        Transaction {
            version: U256::default(),
            hash: H256::default(),
            signature: BoundedVec::default(),
            events: BoundedVec::default(),
            sender_address,
            nonce,
            call_entrypoint,
            selector,
        }
    }
}
