use blockifier::block_context::BlockContext;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;

use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::traits::hash::HasherT;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    Default,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Starknet header definition.
pub struct Header {
    /// The hash of this block’s parent.
    pub parent_block_hash: Felt252Wrapper,
    /// The number (height) of this block.
    pub block_number: u64,
    /// The state commitment after this block.
    pub global_state_root: Felt252Wrapper,
    /// The Starknet address of the sequencer who created this block.
    pub sequencer_address: ContractAddressWrapper,
    /// The time the sequencer created this block before executing transactions
    pub block_timestamp: u64,
    /// The number of transactions in a block
    pub transaction_count: u128,
    /// A commitment to the transactions included in the block
    pub transaction_commitment: Felt252Wrapper,
    /// The number of events
    pub event_count: u128,
    /// A commitment to the events produced in this block
    pub event_commitment: Felt252Wrapper,
    /// The version of the Starknet protocol used when creating this block
    pub protocol_version: u8,
    /// Extraneous data that might be useful for running transactions
    pub extra_data: Option<U256>,
}

impl Header {
    /// Creates a new header.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        parent_block_hash: Felt252Wrapper,
        block_number: u64,
        global_state_root: Felt252Wrapper,
        sequencer_address: ContractAddressWrapper,
        block_timestamp: u64,
        transaction_count: u128,
        transaction_commitment: Felt252Wrapper,
        event_count: u128,
        event_commitment: Felt252Wrapper,
        protocol_version: u8,
        extra_data: Option<U256>,
    ) -> Self {
        Self {
            parent_block_hash,
            block_number,
            global_state_root,
            sequencer_address,
            block_timestamp,
            transaction_count,
            transaction_commitment,
            event_count,
            event_commitment,
            protocol_version,
            extra_data,
        }
    }

    /// Converts to a blockifier BlockContext
    pub fn into_block_context(self, fee_token_address: ContractAddressWrapper, chain_id: ChainId) -> BlockContext {
        // Convert from ContractAddressWrapper to ContractAddress
        let sequencer_address =
            ContractAddress::try_from(StarkFelt::new(self.sequencer_address.into()).unwrap()).unwrap();
        // Convert from ContractAddressWrapper to ContractAddress
        let fee_token_address = ContractAddress::try_from(StarkFelt::new(fee_token_address.into()).unwrap()).unwrap();

        BlockContext {
            chain_id,
            block_number: BlockNumber(self.block_number),
            block_timestamp: BlockTimestamp(self.block_timestamp),
            sequencer_address,
            vm_resource_fee_cost: HashMap::default(),
            fee_token_address,
            invoke_tx_max_n_steps: 1000000,
            validate_max_n_steps: 1000000,
            // FIXME: https://github.com/keep-starknet-strange/madara/issues/329
            gas_price: 10,
        }
    }

    /// Compute the hash of the header.
    #[must_use]
    pub fn hash<H: HasherT>(&self, hasher: H) -> Felt252Wrapper {
        let data: &[Felt252Wrapper] = &[
            self.block_number.into(), // TODO: remove unwrap
            self.global_state_root,
            self.sequencer_address,
            self.block_timestamp.into(),
            self.transaction_count.into(),
            self.transaction_commitment,
            self.event_count.into(),
            self.event_commitment,
            self.protocol_version.into(),
            Felt252Wrapper::ZERO,
            self.parent_block_hash,
        ];

        <H as HasherT>::compute_hash_on_wrappers(&hasher, data)
    }
}
