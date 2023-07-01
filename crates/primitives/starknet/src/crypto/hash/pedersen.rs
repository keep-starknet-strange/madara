//! Pedersen hash module.
use alloc::vec::Vec;

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
    fn compute_hash_on_wrappers(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let hash = Self::compute_hash_on_elements(&data.iter().map(|x| x.0).collect::<Vec<FieldElement>>());
        Felt252Wrapper(hash)
    }

    #[inline(always)]
    fn hash_elements(a: FieldElement, b: FieldElement) -> FieldElement {
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
    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement {
        let mut current_hash = FieldElement::ZERO;

        for item in elements.iter() {
            current_hash = pedersen_hash(&current_hash, item);
        }

        let data_len = FieldElement::from(elements.len());
        pedersen_hash(&current_hash, &data_len)
    }
}

impl DefaultHasher for PedersenHasher {
    fn hasher() -> Self {
        Self::default()
    }
}
