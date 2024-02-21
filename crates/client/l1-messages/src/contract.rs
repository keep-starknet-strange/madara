use mp_felt::{Felt252Wrapper, Felt252WrapperError};
use mp_transactions::HandleL1MessageTransaction;
use starknet_core_contract_client::interfaces::LogMessageToL2Filter;

#[derive(thiserror::Error, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum L1EventToTransactionError {
    #[error("Failed to convert Calldata param from L1 Event: `{0}`")]
    InvalidCalldata(Felt252WrapperError),
    #[error("Failed to convert Contract Address from L1 Event: `{0}`")]
    InvalidContractAddress(Felt252WrapperError),
    #[error("Failed to convert Entrypoint Selector from L1 Event: `{0}`")]
    InvalidEntryPointSelector(Felt252WrapperError),
    #[error("Failed to convert From Address from L1 Event: `{0}`")]
    InvalidFromAddress(Felt252WrapperError),
    #[error("Failed to convert Nonce param from L1 Event: `{0}`")]
    InvalidNonce(Felt252WrapperError),
}

pub fn parse_handle_l1_message_transaction(
    event: LogMessageToL2Filter,
) -> Result<HandleL1MessageTransaction, L1EventToTransactionError> {
    // L1 from address.
    let from_address = Felt252Wrapper::try_from(sp_core::U256::from_big_endian(event.from_address.as_bytes()))
        .map_err(L1EventToTransactionError::InvalidFromAddress)?;

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

    let event_payload: Vec<Felt252Wrapper> = event
        .payload
        .iter()
        .map(|param| Felt252Wrapper::try_from(sp_core::U256(param.0)))
        .collect::<Result<Vec<Felt252Wrapper>, Felt252WrapperError>>()
        .map_err(L1EventToTransactionError::InvalidCalldata)?;

    let mut calldata: Vec<Felt252Wrapper> = Vec::with_capacity(event.payload.len() + 1);
    calldata.push(from_address);
    event_payload.iter().collect_into(&mut calldata);

    Ok(HandleL1MessageTransaction { nonce, contract_address, entry_point_selector, calldata })
}
