//! Cairo Execution Engine pallet custom types.
use frame_support::pallet_prelude::*;

/// Address of a Starknet contract.
pub type StarknetContractAddress = [u8; 32];

/// Sierra program representation.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct StarknetContract {
	/// The identifier of the Sierra program.
	pub contract_address: StarknetContractAddress,
}
