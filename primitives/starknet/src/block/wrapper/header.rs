//! Starknet header definition.
use sp_core::{H256, U256};

#[derive(
	Clone,
	Debug,
	PartialEq,
	Eq,
	codec::Encode,
	codec::Decode,
	scale_info::TypeInfo,
	Default,
	codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
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

	/// Compute the hash of the header.
	/// # TODO
	/// - Implement this function.
	#[must_use]
	pub fn hash(&self) -> H256 {
		H256::from_slice(&[0; 32])
	}
}
