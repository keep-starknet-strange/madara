use starknet_api::core::{ContractAddress, EthAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{L1HandlerTransaction, MessageToL1};

use crate::{MessageL1ToL2, MessageL2ToL1};

pub fn eth_address_to_felt(eth_address: &EthAddress) -> StarkFelt {
    let mut bytes = [0u8; 32];
    // Padding H160 with zeros to 32 bytes (big endian)
    bytes[12..32].copy_from_slice(eth_address.0.as_bytes());
    StarkFelt(bytes)
}

impl From<MessageToL1> for MessageL2ToL1 {
    fn from(message: MessageToL1) -> Self {
        Self { from_address: message.from_address, to_address: message.to_address, payload: message.payload.0 }
    }
}

impl From<L1HandlerTransaction> for MessageL1ToL2 {
    fn from(tx: L1HandlerTransaction) -> Self {
        let mut calldata = (*tx.calldata.0).clone();
        // Source Eth address is always passed as the first calldata arg
        let from_address = ContractAddress(PatriciaKey(calldata.remove(0)));
        Self {
            from_address,
            to_address: tx.contract_address,
            nonce: tx.nonce,
            selector: tx.entry_point_selector,
            payload: calldata,
        }
    }
}
