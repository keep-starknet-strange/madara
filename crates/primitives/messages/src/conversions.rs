use mp_transactions::HandleL1MessageTransaction;
use starknet_api::api_core::{ContractAddress, EthAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::MessageToL1;

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

impl From<HandleL1MessageTransaction> for MessageL1ToL2 {
    fn from(tx: HandleL1MessageTransaction) -> Self {
        let mut calldata = tx.calldata;
        // Source Eth address is always passed as the first calldata arg
        let from_address = ContractAddress(PatriciaKey(StarkFelt::from(calldata.remove(0))));
        Self {
            from_address,
            to_address: tx.contract_address.into(),
            nonce: Nonce(tx.nonce.into()),
            selector: tx.entry_point_selector.into(),
            payload: calldata.into_iter().map(|felt| felt.into()).collect(),
        }
    }
}
