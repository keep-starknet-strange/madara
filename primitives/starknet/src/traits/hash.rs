//! Traits for hashing.

/// A trait for hashing.
pub trait Hasher {
    /// Hashes the given data.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash(&self, data: &[u8]) -> [u8; 32];
}
