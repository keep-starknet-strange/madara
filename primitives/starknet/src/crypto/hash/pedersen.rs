use starknet_crypto::{FieldElement, pedersen_hash};

/// The Pedersen hash function.
/// ### Arguments
/// * `x`: The x coordinate
/// * `y`: The y coordinate
pub fn hash(_data: &[u8]) -> [u8; 32] {
	let field_element = FieldElement::from_byte_slice_be(_data).unwrap();
	let result = FieldElement::to_bytes_be(&pedersen_hash(&FieldElement::ZERO, &field_element));
	result
}
