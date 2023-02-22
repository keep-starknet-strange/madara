use starknet_crypto::FieldElement;
use starknet_ff::FromByteSliceError;

mod pedersen;
mod poseidon;

/// The type of hash function used in the StarkNet protocol.
pub enum HashType {
	Poseidon,
	Pedersen,
}

/// Hashes two field elements using the specified hash function.
/// ### Arguments
/// * `hash_type`: The type of hash function to use.
/// * `x`: The x coordinate
/// * `y`: The y coordinate
/// ### Returns
/// The hash of the two field elements.
pub fn hash(hash_type: HashType, x: &FieldElement, y: &FieldElement) -> FieldElement {
	match hash_type {
		HashType::Poseidon => poseidon::hash(x, y),
		HashType::Pedersen => pedersen::hash(x, y),
	}
}

/// Hashes two byte arrays using the specified hash function.
/// ### Arguments
/// * `hash_type`: The type of hash function to use.
/// * `x`: The x coordinate
/// * `y`: The y coordinate
/// ### Returns
/// The hash of the two field elements.
pub fn hash_bytes(hash_type: HashType, x: &[u8], y: &[u8]) -> Result<[u8; 32], FromByteSliceError> {
	// Convert the byte arrays to field elements.
	let x = FieldElement::from_byte_slice_be(x)?;
	let y = FieldElement::from_byte_slice_be(y)?;
	// Hash the field elements.
	let hash = hash(hash_type, &x, &y);
	// Return the hash as a byte array.
	Ok(hash.to_bytes_be())
}
