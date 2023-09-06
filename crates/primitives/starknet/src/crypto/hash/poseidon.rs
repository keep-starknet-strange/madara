//! Poseidon hash module.
use alloc::vec::Vec;

use starknet_crypto::{poseidon_hash, poseidon_hash_many, FieldElement};

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
        // Calculate the number of 31-byte chunks we'll need, rounding up.
        // (1 byte is used padding to prevent the value of field from being greater than modular)
        // TODO: It is need a way to truncate bytes to fit into values smaller than modular(optimization)
        const CHUNK_SIZE: usize = 31;
        let chunks = data.chunks(CHUNK_SIZE);

        let mut field_element_vector: Vec<FieldElement> = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            // Convert the buffer to a FieldElement and then to a Felt252Wrapper.
            field_element_vector.push(FieldElement::from_byte_slice_be(chunk).unwrap())
        }

        Felt252Wrapper(poseidon_hash_many(&field_element_vector))
    }

    /// Hashes a slice of field elements using the Poseidon hash function.
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
        Self::default()
    }
}

#[test]
fn dynamic_string_hashing() {
    use core::str::FromStr;

    let hasher = PoseidonHasher::hasher();

    let message = "Hello, madara!!. It is poseidon hash.".to_string(); // 37 bytes
    let message = message.as_bytes();
    let hash_value = hasher.hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x05f6f93cec36381735e390c14a9cf3118801f2958a1b3a17d32906b9cbd75b78").unwrap()
        )
    );
}

#[test]
fn short_string_hashing() {
    use core::str::FromStr;

    let hasher = PoseidonHasher::hasher();

    let message = "madara".to_string();
    let message = message.as_bytes();
    let hash_value = hasher.hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x055cda6c81d938e0c009e96b81fac1ffbf00e3100b80ed891faf8b9bdf410fff").unwrap()
        )
    );
}
