//! Starknet state root logic.

use core::marker::PhantomData;

use mp_starknet::crypto::commitment::calculate_contract_state_hash;
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

        // Update contracts trie
        let mut commitments = crate::State::<T>::get();
        let pending_state = crate::PendingState::<T>::iter();

        pending_state.for_each(|(contract_address, storage_diffs)| {
            // Retrieve state trie for this contract.
            let mut state_tree = crate::StorageTries::<T>::get(contract_address).unwrap_or_default();
            // For each smart contract, iterate through storage diffs and update the state trie.
            storage_diffs.into_iter().for_each(|(storage_key, storage_value)| {
                state_tree.set(storage_key, storage_value);
            });

            // Update the state trie for this contract in runtime storage.
            crate::StorageTries::<T>::set(contract_address, Some(state_tree.clone()));

            // We then compute the state root
            // And update the storage trie
            let state_root = state_tree.commit();

            let nonce = crate::Nonces::<T>::get(contract_address);
            let class_hash = crate::ContractClassHashes::<T>::get(contract_address).unwrap_or_default();
            let hash = calculate_contract_state_hash::<T::SystemHash>(class_hash, state_root, nonce);
            commitments.storage_commitment.set(contract_address, hash);

            // Finally update the contracts trie in runtime storage.
            crate::State::<T>::mutate(|state| {
                state.storage_commitment = commitments.clone().storage_commitment;
            });

            crate::PendingState::<T>::remove(contract_address);
        });

        // Compute the final state root
        StateCommitment::<T::SystemHash>::calculate(
            commitments.storage_commitment.commit(),
            commitments.class_commitment.commit(),
        )
    }
}
