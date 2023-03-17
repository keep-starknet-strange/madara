use std::collections::HashMap;

use blockifier::block_context::BlockContext;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;

use super::wrapper::header::Header;

pub trait SerializeBlockContext {
    fn serialize(block_header: Header) -> BlockContext;
}

impl SerializeBlockContext for BlockContext {
    fn serialize(block_header: Header) -> BlockContext {
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
