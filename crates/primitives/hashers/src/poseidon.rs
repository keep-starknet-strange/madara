//! Poseidon hash module.
use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;
use starknet_crypto::{poseidon_hash, poseidon_hash_many, FieldElement};

use super::HasherT;

/// The poseidon hasher.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct PoseidonHasher;

unsafe impl Send for PoseidonHasher {}
unsafe impl Sync for PoseidonHasher {}

impl HasherT for PoseidonHasher {
    /// The Poseidon hash function.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(data: &[u8]) -> Felt252Wrapper {
        // Calculate the number of 31-byte chunks we'll need, rounding up.
        // (1 byte is used padding to prevent the value of field from being greater than modular)
        // TODO: It is need a way to truncate bytes to fit into values smaller than modular(optimization)
        const CHUNK_SIZE: usize = 31;
        let chunks = data.chunks(CHUNK_SIZE);

        let mut field_element_vector: Vec<FieldElement> = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            // It is safe to unwrap here because we know that the chunk size is 31 and the value can not
            // overflow than the field's modulus value. In more detail, the FieldElement Maximum value is 2^251
            // + 17 * 2^192. So the chunk (31 bytes is 248 bits) is smaller than the maximum value (== 2^248 - 1
            // < 2^251 + 17 * 2^192). So it is safe to unwrap here.
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
    fn compute_hash_on_wrappers(data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let data = data.iter().map(|x| x.0).collect::<Vec<_>>();
        Felt252Wrapper(poseidon_hash_many(&data))
    }

    fn hash_elements(a: FieldElement, b: FieldElement) -> FieldElement {
        poseidon_hash(a, b)
    }
    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement {
        poseidon_hash_many(elements)
    }
}

#[test]
fn dynamic_string_hashing() {
    use core::str::FromStr;

    let message = "Hello, madara!!. It is poseidon hash.".to_string(); // 37 bytes
    let message = message.as_bytes();
    let hash_value = PoseidonHasher::hash_bytes(message);

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

    let message = "madara".to_string();
    let message = message.as_bytes();
    let hash_value = PoseidonHasher::hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x055cda6c81d938e0c009e96b81fac1ffbf00e3100b80ed891faf8b9bdf410fff").unwrap()
        )
    );
}
