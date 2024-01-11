use alloc::sync::Arc;

use blockifier::block_context::BlockContext;
use mp_fee::ResourcePrice;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkHash;
use starknet_api::stdlib::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
// #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
/// Starknet header definition.
pub struct Header {
    /// The hash of this block’s parent.
    pub parent_block_hash: StarkHash,
    /// The number (height) of this block.
    pub block_number: u64,
    /// The Starknet address of the sequencer who created this block.
    pub sequencer_address: ContractAddress,
    /// The time the sequencer created this block before executing transactions
    pub block_timestamp: u64,
    /// The number of transactions in a block
    pub transaction_count: u128,
    /// The number of events
    pub event_count: u128,
    /// The version of the Starknet protocol used when creating this block
    pub protocol_version: u8,
    /// l1 gas price for this block
    pub l1_gas_price: ResourcePrice,
    /// Extraneous data that might be useful for running transactions
    pub extra_data: Option<U256>,
}

impl Header {
    /// Creates a new header.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        parent_block_hash: StarkHash,
        block_number: u64,
        sequencer_address: ContractAddress,
        block_timestamp: u64,
        transaction_count: u128,
        event_count: u128,
        protocol_version: u8,
        l1_gas_price: ResourcePrice,
        extra_data: Option<U256>,
    ) -> Self {
        Self {
            parent_block_hash,
            block_number,
            sequencer_address,
            block_timestamp,
            transaction_count,
            event_count,
            protocol_version,
            l1_gas_price,
            extra_data,
        }
    }

    /// Converts to a blockifier BlockContext
    pub fn into_block_context(self, fee_token_address: ContractAddress, chain_id: ChainId) -> BlockContext {
        BlockContext {
            chain_id,
            block_number: BlockNumber(self.block_number),
            block_timestamp: BlockTimestamp(self.block_timestamp),
            sequencer_address: self.sequencer_address,
            vm_resource_fee_cost: Arc::new(HashMap::default()),
            fee_token_address,
            invoke_tx_max_n_steps: 1000000,
            validate_max_n_steps: 1000000,
            // FIXME: https://github.com/keep-starknet-strange/madara/issues/329
            gas_price: 10,
            max_recursion_depth: 50,
        }
    }

    /// Compute the hash of the header.
    pub fn hash<H: HasherT>(&self) -> Felt252Wrapper {
        let data: &[Felt252Wrapper] = &[
            self.block_number.into(),
            self.sequencer_address.0.0.into(),
            self.block_timestamp.into(),
            self.transaction_count.into(),
            self.event_count.into(),
            self.protocol_version.into(),
            Felt252Wrapper::ZERO,
            self.parent_block_hash.into(),
        ];

        H::compute_hash_on_wrappers(data)
    }
}
