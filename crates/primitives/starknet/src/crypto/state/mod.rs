use core::marker::PhantomData;

use crate::execution::types::Felt252Wrapper;
use crate::traits::hash::HasherT;

/// Root of the Merkle-Patricia tree whose leaves are the contracts states
pub type StorageCommitment = Felt252Wrapper;
/// Root of the Merkle-Patricia tree whose leaves are the compiled class hashes
pub type ClassCommitment = Felt252Wrapper;

/// Global Starknet State Commitment
pub struct StateCommitment<T: HasherT>(Felt252Wrapper, PhantomData<T>);

impl<T: HasherT> StateCommitment<T> {
    /// Calculates  global state commitment by combining the storage and class commitment.
    ///
    /// See
    /// <https://github.com/starkware-libs/cairo-lang/blob/12ca9e91bbdc8a423c63280949c7e34382792067/src/starkware/starknet/core/os/state.cairo#L125>
    /// for details.
    pub fn calculate(storage_commitment: StorageCommitment, class_commitment: ClassCommitment) -> Felt252Wrapper {
        if class_commitment == ClassCommitment::ZERO {
            storage_commitment
        } else {
            let global_state_version = Felt252Wrapper::try_from("STARKNET_STATE_V0".as_bytes()).unwrap(); // Unwrap is safu here

            let hash =
                <T>::compute_hash_on_elements(&[global_state_version.0, storage_commitment.0, class_commitment.0]);

            hash.into()
        }
    }
}
