//! Starknet state root logic.

use core::marker::PhantomData;

use mp_starknet::execution::types::Felt252Wrapper;
use sp_core::Get;

use crate::Config;

pub struct IntermediateStateRoot<T>(PhantomData<T>);
impl<T: Config> Get<Felt252Wrapper> for IntermediateStateRoot<T> {
    /// Compute the state root of Starknet and return it.
    /// For now, we just return a dummy state root.
    /// # Returns
    /// * `Felt252Wrapper` - The intermediate state root.
    fn get() -> Felt252Wrapper {
        Felt252Wrapper::ONE
    }
}
