use frame_support::BoundedVec;
use mp_starknet::execution::types::{
    CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper, Felt252Wrapper,
};
use mp_starknet::transaction::types::Transaction;
use scale_codec::{Decode, Encode};
use serde::Deserialize;

use crate::alloc::format;
use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::offchain_worker::OffchainWorkerError;

#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct Message {
    /// Topics of the event.
    pub topics: Vec<String>,
    /// Data of the event.
    pub data: String,
}

#[inline(always)]
pub fn get_messages_events(from_block: u64, to_block: u64) -> String {
    format!(
        "{{
            \"jsonrpc\": \"2.0\",
        \"method\": \"eth_getLogs\",
        \"params\": [
            {{
                \"fromBlock\": \"0x{:x}\",
                \"toBlock\": \"0x{:x}\",
                \"address\": \"0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4\",
                \"topics\": [
                    \"0xdb80dd488acf86d17c747445b0eabb5d57c541d3bd7b6b87af987858e5066b2b\"
                ]
            }}
        ],
        \"id\": 0
    }}",
        from_block, to_block
    )
}

impl Message {
    /// Converts a `Message` into a transaction object.
    pub fn try_into_transaction(&self) -> Result<Transaction, OffchainWorkerError> {
        // Data at least contains a nonce and at some point the fees.
        if self.data.is_empty() {
            return Err(OffchainWorkerError::EmptyData);
        }
        // L2 contract to call.
        let sender_address = match Felt252Wrapper::from_hex_be(self.topics[2].as_str()) {
            Ok(f) => f,
            Err(_) => return Err(OffchainWorkerError::ToTransactionError),
        };

        // Function of the contract to call.
        let selector = match Felt252Wrapper::from_hex_be(self.topics[3].as_str()) {
            Ok(f) => f,
            Err(_) => return Err(OffchainWorkerError::ToTransactionError),
        };

        // Add the from address here so it's directly in the calldata.
        let char_vec = format!("{:}{:}", self.topics[1].trim_start_matches("0x"), self.data.trim_start_matches("0x"))
            .chars()
            .collect::<Vec<char>>();
        // Split the data String into values. (The event Log(a: uin256, b: uin256, c: uin256) logs a single
        // string which is the concatenation of those fields).
        let data_map = char_vec.chunks(64).map(|chunk| chunk.iter().collect::<String>());
        // L1 message nonce.
        let nonce =
            Felt252Wrapper::from_hex_be(&data_map.clone().last().ok_or(OffchainWorkerError::ToTransactionError)?)
                .map_err(|_| OffchainWorkerError::ToTransactionError)?;
        let mut calldata: Vec<Felt252Wrapper> = Vec::new();
        for val in data_map.take(self.data.len() - 2) {
            calldata.push(match Felt252Wrapper::from_hex_be(val.as_str()) {
                Ok(f) => f,
                Err(_) => return Err(OffchainWorkerError::ToTransactionError),
            })
        }
        let calldata = BoundedVec::try_from(calldata).map_err(|_| OffchainWorkerError::ToTransactionError)?;
        let call_entrypoint = CallEntryPointWrapper {
            class_hash: None,
            entrypoint_type: EntryPointTypeWrapper::L1Handler,
            entrypoint_selector: Some(selector),
            calldata,
            storage_address: sender_address,
            caller_address: ContractAddressWrapper::default(),
            // FIXME 710
            initial_gas: Felt252Wrapper::from(0_u8),
        };
        Ok(Transaction { sender_address, nonce, call_entrypoint, ..Transaction::default() })
    }
}

#[cfg(test)]
mod test {
    use frame_support::bounded_vec;
    use mp_starknet::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper};
    use mp_starknet::transaction::types::Transaction;
    use pretty_assertions;

    use super::*;
    use crate::offchain_worker::OffchainWorkerError;

    #[test]
    fn test_try_into_transaction_correct_message_should_work() {
        let felt_one = Felt252Wrapper::ONE;
        let sender_address = felt_one;
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex.clone()], data: hex };
        let expected_tx = Transaction {
            sender_address,
            nonce: Felt252Wrapper::ONE,
            call_entrypoint: CallEntryPointWrapper {
                class_hash: None,
                entrypoint_type: EntryPointTypeWrapper::L1Handler,
                entrypoint_selector: Some(felt_one),
                calldata: bounded_vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE],
                storage_address: felt_one,
                caller_address: ContractAddressWrapper::default(),
                initial_gas: Felt252Wrapper::from(0_u8),
            },
            ..Transaction::default()
        };
        pretty_assertions::assert_eq!(test_message.try_into_transaction().unwrap(), expected_tx);
    }

    #[test]
    fn test_try_into_transaction_incorrect_topic_should_fail() {
        let hex = "0x1".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), "foo".to_owned(), hex.clone()], data: hex };
        assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::ToTransactionError);
    }

    #[test]
    fn test_try_into_transaction_incorrect_selector_in_topic_should_fail() {
        let hex = "0x1".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), "foo".to_owned()], data: hex };
        assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::ToTransactionError);
    }
    #[test]
    fn test_try_into_transaction_empty_data_should_fail() {
        let hex = "0x1".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex], data: "".to_owned() };
        assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::EmptyData);
    }
}
