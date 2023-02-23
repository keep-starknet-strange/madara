//! Starknet block definition.

use super::header::Header;

/// Starknet block definition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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
