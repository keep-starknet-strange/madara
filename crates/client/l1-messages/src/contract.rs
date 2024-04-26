use std::sync::Arc;

use mp_felt::{Felt252Wrapper, Felt252WrapperError};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, L1HandlerTransaction, TransactionVersion};
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
) -> Result<L1HandlerTransaction, L1EventToTransactionError> {
    // L1 from address.
    let from_address = Felt252Wrapper::try_from(sp_core::U256::from_big_endian(event.from_address.as_bytes()))
        .map_err(L1EventToTransactionError::InvalidFromAddress)?
        .into();

    // L2 contract to call.
    let contract_address = Felt252Wrapper::try_from(sp_core::U256(event.to_address.0))
        .map_err(L1EventToTransactionError::InvalidContractAddress)?
        .into();

    // Function of the contract to call.
    let entry_point_selector = Felt252Wrapper::try_from(sp_core::U256(event.selector.0))
        .map_err(L1EventToTransactionError::InvalidEntryPointSelector)?
        .into();

    // L1 message nonce.
    let nonce =
        Felt252Wrapper::try_from(sp_core::U256(event.nonce.0)).map_err(L1EventToTransactionError::InvalidNonce)?.into();

    let event_payload: Vec<_> = event
        .payload
        .iter()
        .map(|param| Felt252Wrapper::try_from(sp_core::U256(param.0)).map(StarkFelt::from))
        .collect::<Result<Vec<_>, Felt252WrapperError>>()
        .map_err(L1EventToTransactionError::InvalidCalldata)?;

    let calldata = {
        let mut calldata: Vec<_> = Vec::with_capacity(event.payload.len() + 1);
        calldata.push(from_address);
        event_payload.iter().collect_into(&mut calldata);

        Calldata(Arc::new(calldata))
    };

    Ok(L1HandlerTransaction {
        nonce,
        contract_address,
        entry_point_selector,
        calldata,
        version: TransactionVersion(StarkFelt::ZERO),
    })
}
