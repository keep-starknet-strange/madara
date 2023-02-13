/// Hash input using Poseidon.
/// # Arguments
/// * `data` - The data to hash.
/// # Returns
/// The hash of the data.
/// # TODO
/// * Integrate `<https://github.com/keep-starknet-strange/poseidon-rs>` when `no_std` support is
///   added.
pub fn poseidon(_data: &[u8]) -> [u8; 32] {
	let output = [0u8; 32];
	output
}
