use ethers::contract::abigen;
use ethers::types::U256;
use mp_felt::Felt252Wrapper;
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
    pub fn try_into_fee(&self) -> Result<Fee, L1MessagesWorkerError> {
        // Check against panic
        // https://docs.rs/ethers/latest/ethers/types/struct.U256.html#method.as_u128
        if self.fee > U256::from_big_endian(&(u128::MAX.to_be_bytes())) {
            Err(L1MessagesWorkerError::ToFeeError)
        } else {
            Ok(Fee(self.fee.as_u128()))
        }
    }
    pub fn try_into_transaction(&self) -> Result<HandleL1MessageTransaction, L1MessagesWorkerError> {
        // L2 contract to call.
        let contract_address = Felt252Wrapper::try_from(sp_core::U256(self.to_address.0))?;

        // Function of the contract to call.
        let entry_point_selector = Felt252Wrapper::try_from(sp_core::U256(self.selector.0))?;

        // L1 message nonce.
        let nonce: u64 = Felt252Wrapper::try_from(sp_core::U256(self.nonce.0))?.try_into()?;

        // Add the from address here so it's directly in the calldata.
        let mut calldata: Vec<Felt252Wrapper> = Vec::from([Felt252Wrapper::try_from(self.from_address.as_bytes())?]);

        for x in &self.payload {
            calldata.push(Felt252Wrapper::try_from(sp_core::U256(x.0))?);
        }

        let tx = HandleL1MessageTransaction { nonce, contract_address, entry_point_selector, calldata };
        Ok(tx)
    }
}
