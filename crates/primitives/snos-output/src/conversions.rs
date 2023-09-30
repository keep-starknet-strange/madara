use mp_transactions::HandleL1MessageTransaction;
use sp_core::H160;
use starknet_api::api_core::{EntryPointSelector, EthAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::MessageToL1;

use crate::{MessageL1ToL2, MessageL2ToL1};

impl From<MessageToL1> for MessageL2ToL1 {
    fn from(message: MessageToL1) -> Self {
        Self { from_address: message.from_address, to_address: message.to_address, payload: message.payload.0 }
    }
}

impl From<HandleL1MessageTransaction> for MessageL1ToL2 {
    fn from(tx: HandleL1MessageTransaction) -> Self {
        let mut calldata = tx.calldata;
        // Source Eth address is always passed as the first calldata arg
        let from_address = StarkFelt::from(calldata.remove(0));
        Self {
            from_address: EthAddress(H160::from_slice(from_address.bytes())),
            to_address: tx.contract_address.into(),
            nonce: Nonce(StarkFelt::from(tx.nonce)),
            selector: EntryPointSelector(StarkFelt::from(tx.entry_point_selector)),
            payload: calldata.into_iter().map(|felt| felt.into()).collect(),
        }
    }
}
