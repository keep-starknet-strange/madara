use ethers::contract::abigen;
use mp_felt::{Felt252Wrapper, Felt252WrapperError};
use mp_transactions::HandleL1MessageTransaction;

use crate::error::L1EventToTransactionError;

abigen!(
    L1Contract,
    r"[
	event LogMessageToL2(address indexed fromAddress, uint256 indexed toAddress, uint256 indexed selector, uint256[] payload, uint256 nonce, uint256 fee)
]"
);

impl TryFrom<LogMessageToL2Filter> for HandleL1MessageTransaction {
    type Error = L1EventToTransactionError;

    fn try_from(event: LogMessageToL2Filter) -> Result<Self, Self::Error> {
        // L2 contract to call.
        let contract_address = Felt252Wrapper::try_from(sp_core::U256(event.to_address.0))
            .map_err(L1EventToTransactionError::InvalidContractAddress)?;

        // Function of the contract to call.
        let entry_point_selector = Felt252Wrapper::try_from(sp_core::U256(event.selector.0))
            .map_err(L1EventToTransactionError::InvalidEntryPointSelector)?;

        // L1 message nonce.
        let nonce: u64 = Felt252Wrapper::try_from(sp_core::U256(event.nonce.0))
            .map_err(L1EventToTransactionError::InvalidNonce)?
            .try_into()
            .map_err(L1EventToTransactionError::InvalidNonce)?;

        let calldata: Vec<Felt252Wrapper> = event
            .payload
            .iter()
            .map(|param| Felt252Wrapper::try_from(sp_core::U256(param.0)))
            .collect::<Result<Vec<Felt252Wrapper>, Felt252WrapperError>>()
            .map_err(L1EventToTransactionError::InvalidCalldata)?;

        Ok(HandleL1MessageTransaction { nonce, contract_address, entry_point_selector, calldata })
    }
}
