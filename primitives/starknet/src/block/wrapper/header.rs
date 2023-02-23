//! Starknet header definition.
use sp_core::U256;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
/// Starknet header definition.
pub struct Header {
	/// The address of the sequencer.
	pub sequencer_address: U256,
}

impl Header {
	/// Creates a new header.
	#[must_use]
	pub fn new(sequencer_address: U256) -> Self {
		Self { sequencer_address }
	}
}
