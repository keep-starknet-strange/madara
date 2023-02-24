//! Starknet pallet custom types.
use sp_runtime::RuntimeDebug;

use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
};

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
	StarknetTransaction,
}
