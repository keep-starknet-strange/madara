//! Starknet state root logic.

use core::marker::PhantomData;

use mp_starknet::crypto::commitment::{calculate_class_commitment_tree_root_hash, calculate_contract_state_hash};
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
        // Compute intermediate roots
        let storage_commitment = calculate_contract_state_hash(hash, root, nonce);
        let class_commitment = calculate_class_commitment_tree_root_hash::<T::SystemHash>(class_hashes);

        // Compute the final state root
        let global_state_root = StateCommitment::<T::SystemHash>::calculate(storage_commitment, class_commitment);

        global_state_root
    }
}
