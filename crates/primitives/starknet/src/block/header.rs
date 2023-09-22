use blockifier::block_context::BlockContext;
use scale_codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
// use frame_support::debug;


use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::traits::hash::HasherT;
use serde::{Deserialize, Serialize};

use starknet_core;


#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, Default, Encode, Decode, TypeInfo, MaxEncodedLen
)]
pub enum BlockStatus {
    #[serde(rename(deserialize = "ABORTED", serialize = "ABORTED"))]
    Aborted,
    #[serde(rename(deserialize = "ACCEPTED_ON_L1", serialize = "ACCEPTED_ON_L1"))]
    AcceptedOnL1,
    #[serde(rename(deserialize = "ACCEPTED_ON_L2", serialize = "ACCEPTED_ON_L2"))]
    #[default]
    AcceptedOnL2,
    #[serde(rename(deserialize = "PENDING", serialize = "PENDING"))]
    Pending,
    #[serde(rename(deserialize = "REVERTED", serialize = "REVERTED"))]
    Reverted,
}

impl From<BlockStatus> for starknet_api::block::BlockStatus {
    fn from(status: BlockStatus) -> Self {
        match status {
            BlockStatus::Aborted => starknet_api::block::BlockStatus::Rejected,
            BlockStatus::AcceptedOnL1 => starknet_api::block::BlockStatus::AcceptedOnL1,
            BlockStatus::AcceptedOnL2 => starknet_api::block::BlockStatus::AcceptedOnL2,
            BlockStatus::Pending => starknet_api::block::BlockStatus::Pending,
            BlockStatus::Reverted => starknet_api::block::BlockStatus::Rejected,
        }
    }
}

impl From<BlockStatus> for starknet_core::types::BlockStatus {
    fn from(status: BlockStatus) -> Self {
        match status {
            BlockStatus::Pending => starknet_core::types::BlockStatus::Pending,
            BlockStatus::AcceptedOnL2 => starknet_core::types::BlockStatus::AcceptedOnL2,
            BlockStatus::AcceptedOnL1 => starknet_core::types::BlockStatus::AcceptedOnL1,
            BlockStatus::Reverted => starknet_core::types::BlockStatus::Rejected, // Assuming Reverted maps to Rejected
            _ => panic!("Unsupported status conversion"), // Handle any additional statuses or provide a default conversion
        }
    }
}

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
    /// The status of this block.
    pub status: BlockStatus,
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
        // status: BlockStatus,
        global_state_root: Felt252Wrapper,
        status: BlockStatus,
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
            status,
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
            vm_resource_fee_cost: Default::default(),
            fee_token_address,
            invoke_tx_max_n_steps: 1000000,
            validate_max_n_steps: 1000000,
            // FIXME: https://github.com/keep-starknet-strange/madara/issues/329
            gas_price: 10,
            max_recursion_depth: 50,
        }
    }

    #[must_use]
    pub fn hash<H: HasherT>(&self, hasher: H) -> Felt252Wrapper {
        let first_07_block = 833u64;
        if self.block_number >= first_07_block {
            let data: &[Felt252Wrapper] = &[
                self.block_number.into(),
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

            // // Print each data for debugging
            // for (i, item) in data.iter().enumerate() {
            //     frame_support::log::info!("data[{}]: {:?}", i, item);
            // }

            <H as HasherT>::compute_hash_on_wrappers(&hasher, data)
        } else {
            let data: &[Felt252Wrapper] = &[
                self.block_number.into(),
                self.global_state_root,
                Felt252Wrapper::ZERO,
                Felt252Wrapper::ZERO,
                self.transaction_count.into(),
                self.transaction_commitment,
                Felt252Wrapper::ZERO,
                Felt252Wrapper::ZERO,
                Felt252Wrapper::ZERO,
                Felt252Wrapper::ZERO,
                Felt252Wrapper::from_hex_be("0x534e5f4d41494e").unwrap(),
                self.parent_block_hash,
            ];

            <H as HasherT>::compute_hash_on_wrappers(&hasher, data)
        }
    }
}
