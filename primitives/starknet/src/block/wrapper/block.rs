//! Starknet block definition.

use super::header::Header;

/// Starknet block definition.
#[derive(
    Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, Default, codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// The block header.
    pub header: Header,
}

impl Block {
    /// Creates a new block.
    #[must_use]
    pub fn new(header: Header) -> Self {
        Self { header }
    }
}
