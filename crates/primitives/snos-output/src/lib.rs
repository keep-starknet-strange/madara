//! StarkNet OS program output primitives.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

#[cfg(feature = "parity-scale-codec")]
mod codec;

mod conversions;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use starknet_api::api_core::{ContractAddress, EntryPointSelector, EthAddress, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StarknetOsOutput {
    /// The state commitment before this block.
    pub prev_state_root: StarkHash,
    /// The state commitment after this block.
    pub new_state_root: StarkHash,
    /// The number (height) of this block.
    pub block_number: u64,
    /// The Starknet chain config hash
    pub config_hash: StarkHash,
    /// List of messages sent to L1 in this block
    pub messages_to_l1: Vec<MessageL2ToL1>,
    /// List of messages from L1 handled in this block
    pub messages_to_l2: Vec<MessageL1ToL2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct MessageL2ToL1 {
    pub from_address: ContractAddress,
    pub to_address: EthAddress,
    pub payload: Vec<StarkFelt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct MessageL1ToL2 {
    pub from_address: EthAddress,
    pub to_address: ContractAddress,
    pub nonce: Nonce,
    pub selector: EntryPointSelector,
    pub payload: Vec<StarkFelt>,
}
