use poseidon::{
	convert::{felts_from_u8s, u8s_from_felts},
	hash_sw8,
	parameters::sw8::GF,
};

/// Hash input using Poseidon with starkware's parameters and a rate of 8.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
fn poseidon(_data: &[u8]) -> [u8; 32] {
	let input = felts_from_u8s::<GF>(&_data);
	let result = u8s_from_felts(&hash_sw8(&input));
	let output: [u8; 32] = match result.try_into() {
		Ok(a) => a,
		Err(_) => panic!("Vec<u8> has the wrong length"),
	};
	output
}

/// Hash input using Pedersen.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
fn pedersen(_data: &[u8]) -> [u8; 32] {
	[0; 32]
}

pub trait Hasher {
	fn hash(data: &[u8]) -> [u8; 32];
}

pub struct Poseidon;
pub struct Pedersen;

impl Hasher for Poseidon {
	fn hash(data: &[u8]) -> [u8; 32] {
		poseidon(data)
	}
}

/// TODO: Implement Pedersen hash
impl Hasher for Pedersen {
	fn hash(_data: &[u8]) -> [u8; 32] {
		pedersen(_data)
	}
}
