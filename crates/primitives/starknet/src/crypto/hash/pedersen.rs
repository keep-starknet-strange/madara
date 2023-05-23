//! Pedersen hash module.
use starknet_crypto::{pedersen_hash, FieldElement};

use crate::traits::hash::{CryptoHasher, Hasher};

/// The Pedersen hash function.
/// ### Arguments
/// * `x`: The x coordinate
/// * `y`: The y coordinate
pub fn hash(data: &[u8]) -> [u8; 32] {
    // For now we use the first 31 bytes of the data as the field element, to avoid any panics.
    // TODO: have proper error handling and think about how to hash efficiently big chunks of data.
    let field_element = FieldElement::from_byte_slice_be(&data[..31]).unwrap();
    FieldElement::to_bytes_be(&pedersen_hash(&FieldElement::ZERO, &field_element))
}

/// The Pedersen hasher.
#[derive(Default)]
pub struct PedersenHasher;

/// The Pedersen hasher implementation.
impl Hasher for PedersenHasher {
    /// Hashes the given data.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash(&self, data: &[u8]) -> [u8; 32] {
        hash(data)
    }

    fn hasher() -> Self {
        Self::default()
    }
}

/// The pedersen CryptoHasher implementation.
impl CryptoHasher for PedersenHasher {
    #[inline(always)]
    fn hash(a: FieldElement, b: FieldElement) -> FieldElement {
        pedersen_hash(&a, &b)
    }

    /// Compute hash on elements, base on the [python implementation](https://github.com/starkware-libs/cairo-lang/blob/12ca9e91bbdc8a423c63280949c7e34382792067/src/starkware/cairo/common/hash_state.py#L6-L15).
    ///
    /// # Arguments
    ///
    /// * `elements` - The elements to hash.
    ///
    /// # Returns
    ///
    /// h(h(h(h(0, data\[0\]), data\[1\]), ...), data\[n-1\]), n).
    #[inline]
    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement {
        if elements.is_empty() {
            <PedersenHasher as CryptoHasher>::hash(FieldElement::ZERO, FieldElement::ZERO)
        } else {
            let hash = elements.iter().fold(FieldElement::ZERO, |a, b| <PedersenHasher as CryptoHasher>::hash(a, *b));
            <PedersenHasher as CryptoHasher>::hash(
                hash,
                FieldElement::from_byte_slice_be(&elements.len().to_be_bytes()).unwrap(),
            )
        }
    }
}
