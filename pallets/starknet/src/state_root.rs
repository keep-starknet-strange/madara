//! Starknet state root logic.

use core::marker::PhantomData;

use sp_core::{Get, U256};
use starknet_crypto::FieldElement;

use crate::Config;

pub struct IntermediateStateRoot<T>(PhantomData<T>);
impl<T: Config> Get<U256> for IntermediateStateRoot<T> {
	/// Compute the state root of Starknet and return it.
	/// For now, we just return a dummy state root.
	/// TODO: Implement this function.
	/// # Returns
	/// * `U256` - The intermediate state root.
	fn get() -> U256 {
		U256::from_big_endian(FieldElement::ONE.to_bytes_be().as_slice())
	}
}
