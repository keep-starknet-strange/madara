use starknet_crypto::FieldElement;

/// The Pedersen hash function.
/// ### Arguments
/// * `x`: The x coordinate
/// * `y`: The y coordinate
pub fn hash(x: &FieldElement, y: &FieldElement) -> FieldElement {
	starknet_crypto::pedersen_hash(x, y)
}
