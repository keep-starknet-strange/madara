use poseidon_hash::{
	convert::{felts_from_u8s, u8s_from_felts},
	hash_sw8,
	parameters::sw8::GF,
};
/// The Poseidon hash function.
pub fn hash(_data: &[u8]) -> [u8; 32] {
	let input = felts_from_u8s::<GF>(&_data);
	let result = u8s_from_felts(&hash_sw8(&input));
	result.try_into().unwrap()
}
