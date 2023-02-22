//! This module contains the hash functions used in the StarkNet protocol.
//use starknet_crypto::FieldElement;
//use starknet_ff::FromByteSliceError;

mod pedersen;
mod poseidon;

/// The type of hash function used in the StarkNet protocol.
pub enum HashType {
	/// The Poseidon hash function.
	Poseidon,
	/// The Pedersen hash function.
	Pedersen,
}

/*
/// Hashes two field elements using the specified hash function.
/// ### Arguments
/// * `hash_type`: The type of hash function to use.
/// * `x`: The x coordinate
/// * `y`: The y coordinate
/// ### Returns
/// The hash of the two field elements.
pub fn hash(hash_type: HashType, data: &[u8]) -> [u8; 32] {
	match hash_type {
		HashType::Poseidon => poseidon::hash(data),
		HashType::Pedersen => pedersen::hash(data),
	}
}

/// Hashes two byte arrays using the specified hash function.
/// ### Arguments
/// * `hash_type`: The type of hash function to use.
/// * `x`: The x coordinate
/// * `y`: The y coordinate
/// ### Returns
/// The hash of the two field elements.
pub fn hash_field(
	hash_type: HashType,
	x: &FieldElement,
	y: &FieldElement,
) -> Result<FieldElement, FromByteSliceError> {
	// Convert the byte arrays to field elements.
	let x = FieldElement::to_bytes_be(x);
	let y = FieldElement::to_bytes_be(y);
	// Hash the field elements.
	let hash = hash(hash_type, [x, y].concat().as_slice());
	// Return the hash as a byte array.
	FieldElement::from_byte_slice_be(&hash)
}
 */
