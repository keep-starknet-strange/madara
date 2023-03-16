use alloc::vec::Vec;

use starknet_crypto::FieldElement;

/// A trait to be able to compute the tx commitment for different tx struct definition.
pub trait Transaction {
    /// Gets the transaction hash.
    fn hash() -> FieldElement;
    /// Gets the signature of the tx.
    fn signature() -> Vec<FieldElement>;
}
