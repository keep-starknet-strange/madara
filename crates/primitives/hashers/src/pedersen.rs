//! Pedersen hash module.
use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::{pedersen_hash, FieldElement};

use super::HasherT;

/// The Pedersen hasher.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct PedersenHasher;

unsafe impl Send for PedersenHasher {}
unsafe impl Sync for PedersenHasher {}

/// The Pedersen hasher implementation.
impl HasherT for PedersenHasher {
    /// The Pedersen hash function.
    /// # Arguments
    /// * `data` - The data to hash.
    /// # Returns
    /// The hash of the data.
    fn hash_bytes(data: &[u8]) -> Felt252Wrapper {
        // Calculate the number of 31-byte chunks we'll need, rounding up.
        // (1 byte is used padding to prevent the value of field from being greater than the field's
        // modulus) TODO: It is need a way to truncate bytes to fit into values smaller than modular
        // (for optimization)
        const CHUNK_SIZE: usize = 31;
        let mut hash_value = FieldElement::ZERO;

        for chunk in data.chunks(CHUNK_SIZE) {
            // It is safe to unwrap here because we know that the chunk size is 31 and the value can not
            // overflow than the field's modulus value. In more detail, the FieldElement Maximum value is 2^251
            // + 17 * 2^192. So the chunk (31 bytes is 248 bits) is smaller than the maximum value (== 2^248 - 1
            // < 2^251 + 17 * 2^192). So it is safe to unwrap here.
            let field_element = FieldElement::from_byte_slice_be(chunk).unwrap();
            hash_value = pedersen_hash(&hash_value, &field_element);
        }

        Felt252Wrapper(hash_value)
    }

    /// Hashes a slice of field elements using the Pedersen hash function.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to hash.
    ///
    /// # Returns
    ///
    /// The hash of the data.
    fn compute_hash_on_wrappers(data: &[Felt252Wrapper]) -> Felt252Wrapper {
        let hash = compute_hash_on_elements(&data.iter().map(|x| x.0).collect::<Vec<FieldElement>>());
        Felt252Wrapper(hash)
    }

    #[inline(always)]
    fn hash_elements(a: FieldElement, b: FieldElement) -> FieldElement {
        pedersen_hash(&a, &b)
    }

    /// Compute hash on elements, taken from [starknet-rs](https://github.com/xJonathanLEI/starknet-rs/blob/master/starknet-core/src/crypto.rs#L25) pending a no_std support.
    ///
    /// # Arguments
    ///
    /// * `elements` - The elements to hash.
    ///
    /// # Returns
    ///
    /// h(h(h(h(0, data\[0\]), data\[1\]), ...), data\[n-1\]), n).
    #[inline]
    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement {
        compute_hash_on_elements(elements)
    }
}

#[test]
fn dynamic_string_hashing() {
    use core::str::FromStr;

    let message = "Hello, madara!!. It is pedersen hash.".to_string(); // 37 bytes
    let message = message.as_bytes();
    let hash_value = PedersenHasher::hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x03501abfd0e0606ecab6702213a03294b81837e4d48232df3c39a62b88cc6f73").unwrap()
        )
    );
}

#[test]
fn short_string_hashing() {
    use core::str::FromStr;

    let message = "madara".to_string();
    let message = message.as_bytes();
    let hash_value = PedersenHasher::hash_bytes(message);

    assert_eq!(
        hash_value,
        Felt252Wrapper(
            FieldElement::from_str("0x04b1b68d0622d978edcef1071b697f003896a8f432d4d5523a2f72ec812591f8").unwrap()
        )
    );
}
