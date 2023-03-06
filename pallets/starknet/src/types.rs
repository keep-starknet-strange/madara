//! Starknet pallet custom types.
use sp_runtime::RuntimeDebug;

use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
};

/// TODO: Replace with a proper type for field element.
/// The address of a Starknet contract.
pub type ContractAddress = [u8; 32];
/// The hash of a Starknet contract class.
pub type ContractClassHash = [u8; 32];

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
	StarknetTransaction,
}
