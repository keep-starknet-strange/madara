//! Traits for hashing.
pub mod pedersen;
pub mod poseidon;

use mp_felt::Felt252Wrapper;
use starknet_crypto::FieldElement;

/// A trait for hashing.
pub trait HasherT {
    /// Hashes the given data.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(data: &[u8]) -> Felt252Wrapper;

    /// Hashes the given data, including the default implementation
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn compute_hash_on_wrappers<I>(data: I) -> Felt252Wrapper
    where
        I: IntoIterator<Item = Felt252Wrapper>,
    {
        // Default implementation
        let hash = Self::compute_hash_on_elements(
            data.into_iter() // Convert the data into an iterator
                .map(|x| x.0), // Map each Felt252Wrapper to its inner FieldElement
        );

        // Wrap the computed hash in a Felt252Wrapper and return it
        Felt252Wrapper(hash)
    }

    /// Hashes the 2 felts sent.
    ///
    /// # Arguments
    ///
    /// * `a` - First element to hash.
    /// * `b` - Second element to hash.
    ///
    /// # Returns
    ///
    /// The hash of the 2 values.
    fn hash_elements(a: FieldElement, b: FieldElement) -> FieldElement;

    /// Computes a hash chain over the data, in the following order:
    /// h(h(h(h(0, data\[0\]), data\[1\]), ...), data\[n-1\]), n).
    /// The hash is initialized with 0 and ends with the data length appended.
    /// The length is appended in order to avoid collisions of the following kind:
    /// H(\[x,y,z\]) = h(h(x,y),z) = H(\[w, z\]) where w = h(x,y).
    ///
    /// # Arguments
    ///
    /// * `elements` - A generic type that implements the Iterator trait.
    ///
    /// # Returns
    ///
    /// The hash of the array.
    fn compute_hash_on_elements<I>(elements: I) -> FieldElement
    where
        I: IntoIterator<Item = FieldElement>;
}
