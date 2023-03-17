use std::collections::HashMap;

use blockifier::block_context::BlockContext;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;

use super::wrapper::header::Header;

/// Trait for serializing objects into a `BlockContext`.
pub trait SerializeBlockContext {
    fn serialize(block_header: Header) -> BlockContext;
}

/// Implementation of the `SerializeBlockContext` trait.
impl SerializeBlockContext for BlockContext {
	/// Serializes a block header into a `BlockContext`.
	///
	/// # Arguments
	///
	/// * `block_header` - The block header to serialize.
	///
	/// # Returns
	///
	/// The serialized block context.
    fn serialize(block_header: Header) -> BlockContext {
		/// TODO: use actual values
        BlockContext {
            chain_id: ChainId("SN_GOERLI".to_string()),
            block_number: BlockNumber(block_header.block_number.as_u64()),
            block_timestamp: BlockTimestamp(block_header.block_timestamp),
            sequencer_address: ContractAddress::try_from(StarkFelt::new(block_header.sequencer_address).unwrap()).unwrap(),
            cairo_resource_fee_weights: HashMap::default(),
			fee_token_address: ContractAddress::default(),
			invoke_tx_max_n_steps: 1000000,
			validate_max_n_steps: 1000000,
        }
    }
}
