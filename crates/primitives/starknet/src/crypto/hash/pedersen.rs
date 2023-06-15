//! Pedersen hash module.
use starknet_crypto::{pedersen_hash, FieldElement};

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::{CryptoHasherT, DefaultHasher, HasherT};

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
    fn hash(&self, data: &[u8]) -> Felt252Wrapper {
        // For now we use the first 31 bytes of the data as the field element, to avoid any panics.
        // TODO: have proper error handling and think about how to hash efficiently big chunks of data.
        let field_element = FieldElement::from_byte_slice_be(&data[..31]).unwrap();
        Felt252Wrapper(pedersen_hash(&FieldElement::ZERO, &field_element))
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
    fn hash_elements(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let mut hash = FieldElement::ZERO;
        for element in data {
            hash = pedersen_hash(&hash, &element.0);
        }

        let data_len = Felt252Wrapper::from(data.len() as u64);
        hash = pedersen_hash(&hash, &data_len.0);

        Felt252Wrapper(hash)
    }
}

impl DefaultHasher for PedersenHasher {
    fn hasher() -> Self {
        Self::default()
    }
}

/// The pedersen CryptoHasher implementation.
impl CryptoHasherT for PedersenHasher {
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
            <PedersenHasher as CryptoHasherT>::hash(FieldElement::ZERO, FieldElement::ZERO)
        } else {
            let hash = elements.iter().fold(FieldElement::ZERO, |a, b| <PedersenHasher as CryptoHasherT>::hash(a, *b));
            <PedersenHasher as CryptoHasherT>::hash(
                hash,
                FieldElement::from_byte_slice_be(&elements.len().to_be_bytes()).unwrap(),
            )
        }
    }
}
