//! L1-L2 messages types definition

use std::vec::Vec;

use starknet_api::core::{ContractAddress, EntryPointSelector, EthAddress, Nonce};
use starknet_api::hash::StarkFelt;

pub mod conversions;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", serde_with::serde_as, derive(serde::Serialize))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub struct MessageL2ToL1 {
    /// The address of the L2 contract sending the message
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub from_address: ContractAddress,
    /// The target L1 address the message is sent to
    pub to_address: EthAddress,
    /// The payload of the message
    #[cfg_attr(feature = "serde", serde_as(as = "Vec<UfeHex>"))]
    pub payload: Vec<StarkFelt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", serde_with::serde_as, derive(serde::Serialize))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
/// Message sent to L2 by calling Starknet smart contract on Ethereum
pub struct MessageL1ToL2 {
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub from_address: ContractAddress,
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub to_address: ContractAddress,
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub nonce: Nonce,
    #[cfg_attr(feature = "serde", serde_as(as = "UfeHex"))]
    pub selector: EntryPointSelector,
    #[cfg_attr(feature = "serde", serde_as(as = "Vec<UfeHex>"))]
    pub payload: Vec<StarkFelt>,
}
