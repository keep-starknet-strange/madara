use poseidon::{
	convert::{felts_from_u8s, u8s_from_felts},
	hash_sw8 as hash,
	parameters::sw8::GF,
};

/// Hash input using Poseidon with starkware's parameters and a rate of 8.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
pub fn poseidon(_data: &[u8]) -> [u8; 32] {
	let input = felts_from_u8s::<GF>(&_data);
	let result = u8s_from_felts(&hash(&input));
	let output: [u8; 32] = match result.try_into() {
		Ok(a) => a,
		Err(_) => panic!("Vec<u8> has the wrong length"),
	};
	output
}
