use frame_support::BoundedVec;
use mp_starknet::execution::{CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::Transaction;
use sp_core::{H256, U256};

use crate::pallet::alloc::borrow::ToOwned;
use crate::pallet::alloc::format;
use crate::pallet::alloc::string::String;
use crate::pallet::alloc::vec::Vec;
use crate::types::{Message, OffchainWorkerError};

pub const LAST_FINALIZED_BLOCK_QUERY: &str =
    r#"{"jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["finalized", true], "id": 0}"#;

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
    /// Converts a hex `String` into a byte slice little endian.
    ///
    /// # Arguments
    /// * `s` - The hex string
    ///
    /// # Returns
    ///
    /// A fixed size byte slice.
    pub fn decode_hex_be(s: &str) -> Result<[u8; 32], OffchainWorkerError> {
        let s = s.trim_start_matches("0x");
        let s = if s.len() % 2 != 0 { format!("0{:}", s) } else { s.to_owned() };
        let mut res = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for (id, i) in (0..s.len()).step_by(2).enumerate() {
            res[id] = u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| OffchainWorkerError::HexDecodeError)?;
        }

        Ok(res)
    }

    /// Converts a `Message` into a transaction object.
    pub fn try_into_transaction(&self) -> Result<Transaction, OffchainWorkerError> {
        // Data at least contains a nonce and at some point the fees.
        if self.data.is_empty() {
            return Err(OffchainWorkerError::EmptyData);
        }
        // L2 contract to call.
        let sender_address = Self::decode_hex_be(&self.topics[2])?;
        // Function of the contract to call.
        let selector = H256::from_slice(&Self::decode_hex_be(&self.topics[3])?);
        // Add the from address here so it's directly in the calldata.
        let char_vec = format!("{:}{:}", self.topics[1].trim_start_matches("0x"), self.data.trim_start_matches("0x"))
            .chars()
            .collect::<Vec<char>>();
        // Split the data String into values. (The event Log(a: uin256, b: uin256, c: uin256) logs a single
        // string which is the concatenation of those fields).
        let data_map = char_vec.chunks(64).map(|chunk| chunk.iter().collect::<String>());
        // L1 message nonce.
        let nonce = U256::from_str_radix(&data_map.clone().last().ok_or(OffchainWorkerError::ToTransactionError)?, 16)
            .map_err(|_| OffchainWorkerError::ToTransactionError)?;
        let mut calldata = Vec::new();
        for val in data_map.take(self.data.len() - 2) {
            calldata.push(U256::from_big_endian(&Self::decode_hex_be(&val)?))
        }
        let calldata = BoundedVec::try_from(calldata).map_err(|_| OffchainWorkerError::ToTransactionError)?;
        let call_entrypoint = CallEntryPointWrapper {
            class_hash: None,
            entrypoint_type: EntryPointTypeWrapper::L1Handler,
            entrypoint_selector: Some(selector),
            calldata,
            storage_address: sender_address,
            caller_address: ContractAddressWrapper::default(),
        };
        Ok(Transaction {
            version: U256::default(),
            hash: H256::default(),
            signature: BoundedVec::default(),
            events: BoundedVec::default(),
            sender_address,
            nonce,
            call_entrypoint,
            contract_class: None,
        })
    }
}

#[cfg(test)]
mod test {
    use frame_support::{bounded_vec, BoundedVec};
    use mp_starknet::execution::{CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper};
    use mp_starknet::transaction::types::Transaction;
    use pretty_assertions;
    use sp_core::{H256, U256};
    use test_case::test_case;

    use crate::types::{Message, OffchainWorkerError};

    #[test_case("0x01", true, Some([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]))]
    #[test_case("02", true, Some([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]))]
    #[test_case("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", true, Some([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]))]
    #[test_case("foo", false, None)]
    fn test_decode_hex(value: &str, should_success: bool, result: Option<[u8; 32]>) {
        let res = Message::decode_hex_be(value);
        if should_success {
            assert!(res.is_ok());
            pretty_assertions::assert_eq!(res.unwrap(), result.unwrap())
        } else {
            assert!(res.is_err());
            assert_eq!(res.unwrap_err(), OffchainWorkerError::HexDecodeError);
        }
    }

    #[test]
    fn test_try_into_transaction_correct_message_should_work() {
        let sender_address = H256::from_low_u64_be(1).to_fixed_bytes();
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex.clone()], data: hex };
        let expected_tx = Transaction {
            version: U256::default(),
            hash: H256::default(),
            signature: BoundedVec::default(),
            events: BoundedVec::default(),
            sender_address,
            nonce: U256::from(1),
            call_entrypoint: CallEntryPointWrapper {
                class_hash: None,
                entrypoint_type: EntryPointTypeWrapper::L1Handler,
                entrypoint_selector: Some(H256::from_low_u64_be(1)),
                calldata: bounded_vec![U256::from(1), U256::from(1)],
                storage_address: H256::from_low_u64_be(1).to_fixed_bytes(),
                caller_address: ContractAddressWrapper::default(),
            },
            contract_class: None,
        };
        pretty_assertions::assert_eq!(test_message.try_into_transaction().unwrap(), expected_tx);
    }

    #[test]
    fn test_try_into_transaction_incorrect_topic_should_fail() {
        let hex = "0x1".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), "foo".to_owned(), hex.clone()], data: hex };
        assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::HexDecodeError);
    }
    #[test]
    fn test_try_into_transaction_empty_data_should_fail() {
        let hex = "0x1".to_owned();
        let test_message: Message =
            Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex], data: "".to_owned() };
        assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::EmptyData);
    }
}
