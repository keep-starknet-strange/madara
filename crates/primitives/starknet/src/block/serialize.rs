use blockifier::block_context::BlockContext;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;

use crate::alloc::string::ToString;
use crate::block::header::Header;
use crate::execution::ContractAddressWrapper;

/// Trait for serializing objects into a `BlockContext`.
pub trait SerializeBlockContext {
    /// The type returned in the event of a serialization error.
    type Error;
    /// Serializes a block header into a `BlockContext`.
    fn try_serialize(
        block_header: Header,
        fee_token_address: ContractAddressWrapper,
    ) -> Result<BlockContext, Self::Error>;
}

/// Errors that can occur when serializing a block context.
#[derive(Debug)]
pub enum BlockSerializationError {
    /// Error when serializing the sequencer address.
    SequencerAddressError,
    /// Error when serializing the fee token address.
    FeeTokenAddressError,
}

/// Implementation of the `SerializeBlockContext` trait.
impl SerializeBlockContext for BlockContext {
    type Error = BlockSerializationError;
    /// Serializes a block header into a `BlockContext`.
    ///
    /// # Arguments
    ///
    /// * `block_header` - The block header to serialize.
    ///
    /// # Returns
    ///
    /// The serialized block context.
    /// TODO: use actual values
    fn try_serialize(
        block_header: Header,
        fee_token_address: ContractAddressWrapper,
    ) -> Result<BlockContext, Self::Error> {
        // Try to serialize the sequencer address.
        let sequencer_address = ContractAddress::try_from(
            StarkFelt::new(block_header.sequencer_address)
                .map_err(|_| BlockSerializationError::SequencerAddressError)?,
        )
        .map_err(|_| BlockSerializationError::SequencerAddressError)?;
        // Try to serialize the fee token address.
        let fee_token_address = ContractAddress::try_from(
            StarkFelt::new(fee_token_address).map_err(|_| BlockSerializationError::FeeTokenAddressError)?,
        )
        .map_err(|_| BlockSerializationError::FeeTokenAddressError)?;

        Ok(BlockContext {
            chain_id: ChainId("SN_GOERLI".to_string()),
            block_number: BlockNumber(block_header.block_number.as_u64()),
            block_timestamp: BlockTimestamp(block_header.block_timestamp),
            sequencer_address,
            cairo_resource_fee_weights: HashMap::default(),
            fee_token_address,
            invoke_tx_max_n_steps: 1000000,
            validate_max_n_steps: 1000000,
            gas_price: 0,
        })
    }
}

// Tests for the `SerializeBlockContext` trait.
#[cfg(test)]
mod tests {
    use hex::FromHex;

    use super::*;

    #[test]
    fn test_try_serialize() {
        let sequencer_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000FF").unwrap();
        // Create a block header.
        let block_header =
            Header { block_number: 1.into(), block_timestamp: 1, sequencer_address, ..Default::default() };
        // Create a fee token address.
        let fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap();
        // Try to serialize the block header.
        let block_context = BlockContext::try_serialize(block_header, fee_token_address).unwrap();
        let expected_sequencer_address = ContractAddress::try_from(StarkFelt::new(sequencer_address).unwrap()).unwrap();
        let expected_fee_token_address = ContractAddress::try_from(StarkFelt::new(fee_token_address).unwrap()).unwrap();
        // Check that the block context was serialized correctly.
        assert_eq!(block_context.block_number, BlockNumber(1));
        assert_eq!(block_context.block_timestamp, BlockTimestamp(1));
        assert_eq!(block_context.sequencer_address, expected_sequencer_address);
        assert_eq!(block_context.fee_token_address, expected_fee_token_address);
    }

    #[test]
    fn test_try_serialize_invalid_sequencer_address() {
        // Use a value greater than the PATRICIA_KEY_UPPER_BOUND
        // (0x0800000000000000000000000000000000000000000000000000000000000000)
        let sequencer_address =
            <[u8; 32]>::from_hex("0800000000000000000000000000000000000000000000000000000000000001").unwrap();
        // Create a block header.
        let block_header =
            Header { block_number: 1.into(), block_timestamp: 1, sequencer_address, ..Default::default() };
        // Create a fee token address.
        let fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap();
        // Try to serialize the block header.
        let block_context_result = BlockContext::try_serialize(block_header, fee_token_address);
        // Check that the result is an error.
        assert!(block_context_result.is_err());
    }

    #[test]
    fn test_try_serialize_invalid_fee_token_address() {
        let sequencer_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000FF").unwrap();
        // Create a block header.
        let block_header =
            Header { block_number: 1.into(), block_timestamp: 1, sequencer_address, ..Default::default() };
        // Create a fee token address.
        // Use a value greater than the PATRICIA_KEY_UPPER_BOUND
        // (0x0800000000000000000000000000000000000000000000000000000000000000)
        let fee_token_address =
            <[u8; 32]>::from_hex("0800000000000000000000000000000000000000000000000000000000000001").unwrap();
        // Try to serialize the block header.
        let block_context_result = BlockContext::try_serialize(block_header, fee_token_address);
        // Check that the result is an error.
        assert!(block_context_result.is_err());
    }
}
