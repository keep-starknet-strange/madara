use ethers::contract::abigen;
use ethers::types::U256;
use mp_felt::{Felt252Wrapper, Felt252WrapperError};
use mp_transactions::HandleL1MessageTransaction;
use starknet_api::transaction::Fee;

use crate::error::L1MessagesWorkerError;

abigen!(
    L1Contract,
    r"[
	event LogMessageToL2(address indexed fromAddress, uint256 indexed toAddress, uint256 indexed selector, uint256[] payload, uint256 nonce, uint256 fee)
]"
);

impl LogMessageToL2Filter {
    pub fn try_get_fee(&self) -> Result<Fee, L1MessagesWorkerError> {
        // Check against panic
        // https://docs.rs/ethers/latest/ethers/types/struct.U256.html#method.as_u128
        if self.fee > U256::from_big_endian(&(u128::MAX.to_be_bytes())) {
            Err(L1MessagesWorkerError::ToFeeError)
        } else {
            Ok(Fee(self.fee.as_u128()))
        }
    }
}

impl TryFrom<&LogMessageToL2Filter> for HandleL1MessageTransaction {
    type Error = L1MessagesWorkerError;

    fn try_from(event: &LogMessageToL2Filter) -> Result<Self, Self::Error> {
        // L2 contract to call.
        let contract_address = Felt252Wrapper::try_from(sp_core::U256(event.to_address.0))?;

        // Function of the contract to call.
        let entry_point_selector = Felt252Wrapper::try_from(sp_core::U256(event.selector.0))?;

        // L1 message nonce.
        let nonce: u64 = Felt252Wrapper::try_from(sp_core::U256(event.nonce.0))?.try_into()?;

        let calldata: Vec<Felt252Wrapper> = event
            .payload
            .iter()
            .map(|param| Felt252Wrapper::try_from(sp_core::U256(param.0)))
            .collect::<Result<Vec<Felt252Wrapper>, Felt252WrapperError>>()?;

        Ok(HandleL1MessageTransaction { nonce, contract_address, entry_point_selector, calldata })
    }
}
