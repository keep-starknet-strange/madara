use poseidon::{
	convert::{felts_from_u8s, u8s_from_felts},
	hash_sw8,
	parameters::sw8::GF,
};
use starknet_crypto::pedersen_hash;

/// Hash input using Poseidon with starkware's parameters and a rate of 8.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
fn poseidon(_data: &[u8]) -> [u8; 32] {
	let input = felts_from_u8s::<GF>(&_data);
	let result = u8s_from_felts(&hash_sw8(&input));
	result.into()
}

/// Hash input using Pedersen.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
fn pedersen(_data: &[u8]) -> [u8; 32] {
	pedersen_hash(0, _data).into()
}

pub trait Hasher {
	fn hash(data: &[u8]) -> [u8; 32];
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub struct Poseidon;
#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub struct Pedersen;

impl Hasher for Poseidon {
	fn hash(data: &[u8]) -> [u8; 32] {
		poseidon(data)
	}
}

impl Hasher for Pedersen {
	fn hash(_data: &[u8]) -> [u8; 32] {
		pedersen(_data)
	}
}
