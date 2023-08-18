//! Pedersen hash module.
use alloc::vec::Vec;
use core::cmp;

use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::{pedersen_hash, FieldElement};

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::{DefaultHasher, HasherT};

/// The Pedersen hasher.
#[derive(Clone, Copy, Default, scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct PedersenHasher;

/// The Pedersen hasher implementation.
impl HasherT for PedersenHasher {
    /// The Pedersen hash function.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(&self, data: &[u8]) -> Felt252Wrapper {
        // Calculate the number of 31-byte chunks we'll need, rounding up.
        // (1 byte is used padding to prevent the value of field from being greater than modular)
        // TODO: It is need a way to truncate bytes to fit into values smaller than modular (optimization)
        let chunks = (data.len() + 30) / 31;

        let mut hash_value = FieldElement::ZERO;

        for i in 0..chunks {
            let start = i * 31;
            let end = cmp::min(start + 31, data.len());

            // Create a buffer for our 32-byte chunk.
            let mut buffer = [0u8; 32];
            buffer[1..end - start + 1].copy_from_slice(&data[start..end]);

            // Convert the buffer to a FieldElement and then to a Felt252Wrapper.
            let field_element = FieldElement::from_bytes_be(&buffer).unwrap();
            hash_value = pedersen_hash(&hash_value, &field_element);
        }

        Felt252Wrapper(hash_value)
    }

    /// Hashes a slice of field elements using the Pedersen hash function.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to hash.
    ///
    /// # Returns
    ///
    /// The hash of the data.
    fn compute_hash_on_wrappers(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let hash = compute_hash_on_elements(&data.iter().map(|x| x.0).collect::<Vec<FieldElement>>());
        Felt252Wrapper(hash)
    }

    #[inline(always)]
    fn hash_elements(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        pedersen_hash(&a, &b)
    }

    /// Compute hash on elements, taken from [starknet-rs](https://github.com/xJonathanLEI/starknet-rs/blob/master/starknet-core/src/crypto.rs#L25) pending a no_std support.
    ///
    /// # Arguments
    ///
    /// * `elements` - The elements to hash.
    ///
    /// # Returns
    ///
    /// h(h(h(h(0, data\[0\]), data\[1\]), ...), data\[n-1\]), n).
    #[inline]
    fn compute_hash_on_elements(&self, elements: &[FieldElement]) -> FieldElement {
        compute_hash_on_elements(elements)
    }
}

impl DefaultHasher for PedersenHasher {
    fn hasher() -> Self {
        Self::default()
    }
}

#[test]
fn dynamic_string_hashing() {
    use core::str::FromStr;

    let hasher = PedersenHasher::hasher();

    let message = format!("Hello, madara!!. It is pedersen hash."); // 37 bytes
    let message = message.as_bytes();
    let hash_value = hasher.hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x05a76d229982b7175a4da818ceec34c08690af7db687fa036838beccc87e7ed1").unwrap()
        )
    );
}
