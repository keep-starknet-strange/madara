//! StarkNet OS program output primitives.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

mod codec;
mod conversions;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use starknet_api::hash::StarkFelt;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
/// Main part of Starknet OS program output
pub struct StarknetOsOutput {
    /// The state commitment before this block.
    pub prev_state_root: StarkFelt,
    /// The state commitment after this block.
    pub new_state_root: StarkFelt,
    /// The number (height) of this block.
    pub block_number: StarkFelt,
    /// The hash of this block.
    pub block_hash: StarkFelt,
    /// The Starknet chain config hash
    pub config_hash: StarkFelt,
    /// List of messages sent to L1 in this block
    pub messages_to_l1: Vec<MessageL2ToL1>,
    /// List of messages from L1 handled in this block
    pub messages_to_l2: Vec<MessageL1ToL2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
/// Message sent to L1 by invoking according Statknet syscall
pub struct MessageL2ToL1 {
    pub from_address: StarkFelt,
    pub to_address: StarkFelt,
    pub payload: Vec<StarkFelt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
/// Message sent to L2 by calling Starknet smart contract on Ethereum
pub struct MessageL1ToL2 {
    pub from_address: StarkFelt,
    pub to_address: StarkFelt,
    pub nonce: StarkFelt,
    pub selector: StarkFelt,
    pub payload: Vec<StarkFelt>,
}
