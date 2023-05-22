//! This module contains the hash functions used in the StarkNet protocol.
use crate::execution::felt252_wrapper::Felt252Wrapper;

pub mod pedersen;
pub mod poseidon;

/// The type of hash function used in the StarkNet protocol.
pub enum HashType {
    /// The Poseidon hash function.
    Poseidon,
    /// The Pedersen hash function.
    Pedersen,
}

/// Hashes two field elements using the specified hash function.
/// # Arguments
///
/// * `hash_type`: The type of hash function to use.
/// * `x`: The x coordinate
/// * `y`: The y coordinate
///
/// # Returns
///
/// The hash of the two field elements.
pub fn hash(hash_type: HashType, data: &[u8]) -> Felt252Wrapper {
    match hash_type {
        HashType::Poseidon => poseidon::hash(data),
        HashType::Pedersen => pedersen::hash(data),
    }
}
