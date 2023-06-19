//! Starknet state root logic.

use core::marker::PhantomData;

use mp_starknet::crypto::state::StateCommitment;
use mp_starknet::execution::types::Felt252Wrapper;
use sp_core::Get;

use crate::Config;

pub struct IntermediateStateRoot<T>(PhantomData<T>);
impl<T: Config> Get<Felt252Wrapper> for IntermediateStateRoot<T> {
    /// Compute the state root of Starknet and return it.
    /// # Returns
    /// * `Felt252Wrapper` - The intermediate state root.
    fn get() -> Felt252Wrapper {
        // If state root is disabled, return one.
        if !T::EnableStateRoot::get() {
            return Felt252Wrapper::ONE;
        }

        // Get commitmment trees.
        let mut commitments = crate::State::<T>::get();

        // Compute the final state root
        StateCommitment::<T::SystemHash>::calculate(
            commitments.storage_commitment.commit(),
            commitments.class_commitment.commit(),
        )
    }
}
