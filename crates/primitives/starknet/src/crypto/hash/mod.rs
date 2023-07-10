//! This module contains the hash functions used in the StarkNet protocol.
use starknet_ff::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::HasherT;
use crate::traits::ThreadSafeCopy;

pub mod pedersen;
pub mod poseidon;

/// Available hashers in the StarkNet protocol.
#[derive(Clone, Copy, scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Hasher {
    /// The Pedersen hash function.
    Pedersen(pedersen::PedersenHasher),
    /// The Poseidon hash function.
    Poseidon(poseidon::PoseidonHasher),
}

impl ThreadSafeCopy for Hasher {}

/// Implement the `HasherT` trait for the `Hasher` enum.
impl HasherT for Hasher {
    fn hash_bytes(&self, data: &[u8]) -> Felt252Wrapper {
        match self {
            Self::Pedersen(p) => p.hash_bytes(data),
            Self::Poseidon(p) => p.hash_bytes(data),
        }
    }

    fn compute_hash_on_wrappers(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        match self {
            Self::Pedersen(p) => p.compute_hash_on_wrappers(data),
            Self::Poseidon(p) => p.compute_hash_on_wrappers(data),
        }
    }

    fn hash_elements(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        match self {
            Self::Pedersen(p) => p.hash_elements(a, b),
            Self::Poseidon(p) => p.hash_elements(a, b),
        }
    }

    fn compute_hash_on_elements(&self, elements: &[FieldElement]) -> FieldElement {
        match self {
            Self::Pedersen(p) => p.compute_hash_on_elements(elements),
            Self::Poseidon(p) => p.compute_hash_on_elements(elements),
        }
    }
}

impl Default for Hasher {
    fn default() -> Self {
        // To avoid ambiguity, the user has to explicitly choose a hasher.
        unreachable!("Hasher::default() should never be called");
    }
}

/// Implement the `From` trait for the `Hasher` enum.
macro_rules! into_hasher {
    ($(($hash_function:ident, $inner:ty)),+ ) => {
        $(
            impl From<$inner> for Hasher {
                fn from(item: $inner) -> Self {
                    Hasher::$hash_function(item)
                }
            }
        )+
    };
}

into_hasher! {
    (Pedersen, pedersen::PedersenHasher),
    (Poseidon, poseidon::PoseidonHasher)
}

/// Hashes a slice of bytes using the given hash function.
/// # Arguments
///
/// * `hasher`: The hash function to use.
/// * `data`: The data to hash.
///
/// # Returns
///
/// The hash of the data.
pub fn hash(hasher: Hasher, data: &[u8]) -> Felt252Wrapper {
    match hasher {
        Hasher::Pedersen(p) => p.hash_bytes(data),
        Hasher::Poseidon(p) => p.hash_bytes(data),
    }
}
