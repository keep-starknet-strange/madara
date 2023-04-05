//! Poseidon hash module.
use poseidon_hash::convert::{felts_from_u8s, u8s_from_felts};
use poseidon_hash::hash_sw8;
use poseidon_hash::parameters::sw8::GF;
use starknet_crypto::FieldElement;

use crate::traits::hash::CryptoHasher;

/// The poseidon hasher.
#[derive(Default)]
pub struct PoseidonHasher;

/// The Poseidon hash function.
pub fn hash(_data: &[u8]) -> [u8; 32] {
    let input = felts_from_u8s::<GF>(_data);
    let result = u8s_from_felts(&hash_sw8(&input));
    result.try_into().unwrap()
}

/// The poseidon CryptoHasher implementation.
impl CryptoHasher for PoseidonHasher {
    fn hash(a: FieldElement, b: FieldElement) -> FieldElement {
        let input = felts_from_u8s::<GF>(&[a.to_bytes_be(), b.to_bytes_be()].concat());
        FieldElement::from_byte_slice_be(&u8s_from_felts(&poseidon_hash::hash_sw8(&input))).unwrap()
    }
    fn compute_hash_on_elements(_elements: &[FieldElement]) -> FieldElement {
        if _elements.is_empty() {
            <PoseidonHasher as CryptoHasher>::hash(FieldElement::ZERO, FieldElement::ZERO)
        } else {
            let hash = _elements.iter().fold(FieldElement::ZERO, |a, b| <PoseidonHasher as CryptoHasher>::hash(a, *b));
            <PoseidonHasher as CryptoHasher>::hash(
                hash,
                FieldElement::from_byte_slice_be(&_elements.len().to_be_bytes()).unwrap(),
            )
        }
    }
}
