//! Poseidon hash module.
use alloc::vec::Vec;

use poseidon_hash::convert::{felts_from_u8s, u8s_from_felts};
use poseidon_hash::hash_sw8;
use poseidon_hash::parameters::sw8::GF;
use starknet_crypto::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::{CryptoHasherT, DefaultHasher, HasherT};

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
    fn hash(&self, data: &[u8]) -> Felt252Wrapper {
        let input = felts_from_u8s::<GF>(data);
        let binding = u8s_from_felts(&hash_sw8(&input));
        let result = binding.as_slice();
        result.try_into().unwrap() // TODO: remove unwrap
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
    fn hash_elements(&self, data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let input = felts_from_u8s::<GF>(&data.iter().flat_map(|x| x.0.to_bytes_be()).collect::<Vec<u8>>());
        let binding = u8s_from_felts(&hash_sw8(&input));
        let result = binding.as_slice();
        result.try_into().unwrap() // TODO: remove unwrap
    }
}

impl DefaultHasher for PoseidonHasher {
    fn hasher() -> Self {
        Self::default()
    }
}

/// The poseidon CryptoHasher implementation.
impl CryptoHasherT for PoseidonHasher {
    fn hash(a: FieldElement, b: FieldElement) -> FieldElement {
        let input = felts_from_u8s::<GF>(&[a.to_bytes_be(), b.to_bytes_be()].concat());
        FieldElement::from_byte_slice_be(&u8s_from_felts(&poseidon_hash::hash_sw8(&input))).unwrap()
    }
    fn compute_hash_on_elements(_elements: &[FieldElement]) -> FieldElement {
        todo!()
    }
}
