//! Traits for hashing.

use starknet_crypto::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;

/// A trait for hashing.
pub trait HasherT: Default {
    /// Hashes the given data.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(&self, data: &[u8]) -> Felt252Wrapper;

    // Hashes the given data.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn compute_hash_on_wrappers(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper;

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
    /// * `elements` - The array to hash.
    ///
    /// # Returns
    ///
    /// The hash of the array.
    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement;
}

/// A trait for default hashing instance.
pub trait DefaultHasher {
    /// Get Hasher default instance.
    fn hasher() -> Self;
}
