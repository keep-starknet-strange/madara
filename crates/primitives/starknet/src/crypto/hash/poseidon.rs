//! Poseidon hash module.
use alloc::vec::Vec;
use core::cmp;

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
        let chunks = (data.len() + 30) / 31;

        let mut data_vectors: Vec<Felt252Wrapper> = Vec::with_capacity(chunks);

        for i in 0..chunks {
            let start = i * 31;
            let end = cmp::min(start + 31, data.len());

            // Create a buffer for our 32-byte chunk.
            let mut buffer = [0u8; 32];
            buffer[1..end - start + 1].copy_from_slice(&data[start..end]);

            // Convert the buffer to a FieldElement and then to a Felt252Wrapper.
            let field_element = FieldElement::from_bytes_be(&buffer).unwrap();
            data_vectors.push(Felt252Wrapper(field_element))
        }

        self.compute_hash_on_wrappers(&data_vectors)
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

    let message = format!("Hello, madara!!. It is poseidon hash."); // 37 bytes
    let message = message.as_bytes();
    let hash_value = hasher.hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x029b80231608c1cfcd7ff4aa3e7148fae5c16bf3c6e1b61a1034de7c0ac8469a").unwrap()
        )
    );
}
