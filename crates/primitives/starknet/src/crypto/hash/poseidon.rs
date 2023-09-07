//! Poseidon hash module.
use alloc::vec::Vec;

use starknet_crypto::{poseidon_hash, poseidon_hash_many, poseidon_hash_single, FieldElement};

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::{DefaultHasher, HasherT};

/// The poseidon hasher.
#[derive(Clone, Copy, Default, scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct PoseidonHasher;

impl HasherT for PoseidonHasher {
    /// The Poseidon hash function.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(&self, data: &[u8]) -> Felt252Wrapper {
        let data = FieldElement::from_byte_slice_be(data).unwrap();
        Felt252Wrapper(poseidon_hash_single(data))
    }

    /// Hashes a slice of field elements using the Poseido hash function.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to hash.
    ///
    /// # Returns
    ///
    /// The hash of the data.
    fn compute_hash_on_wrappers(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let data = data.iter().map(|x| x.0).collect::<Vec<_>>();
        Felt252Wrapper(poseidon_hash_many(&data))
    }

    fn hash_elements(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        poseidon_hash(a, b)
    }
    fn compute_hash_on_elements(&self, elements: &[FieldElement]) -> FieldElement {
        poseidon_hash_many(elements)
    }
}

impl DefaultHasher for PoseidonHasher {
    fn hasher() -> Self {
        Self
    }
}
