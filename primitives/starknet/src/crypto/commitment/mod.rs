use alloc::vec::Vec;

use bitvec::vec::BitVec;
use sp_core::H256;
use starknet_crypto::FieldElement;

use super::hash::pedersen;
use super::merkle_patricia_tree::merkle_tree::MerkleTree;
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::traits::hash::CryptoHasher;

/// A Patricia Merkle tree with height 64 used to compute transaction and event commitments.
///
/// According to the [documentation](https://docs.starknet.io/docs/Blocks/header/#block-header)
/// the commitment trees are of height 64, because the key used is the 64 bit representation
/// of the index of the transaction / event within the block.
///
/// The tree height is 64 in our case since our set operation takes u64 index values.
struct CommitmentTree {
    tree: MerkleTree<(), PedersenHasher>,
}

impl Default for CommitmentTree {
    fn default() -> Self {
        Self { tree: MerkleTree::empty((), 64) }
    }
}

impl CommitmentTree {
    pub fn set(&mut self, index: u64, value: FieldElement) {
        let key = index.to_be_bytes();
        self.tree.set(&BitVec::from(key.to_vec()), value)
    }

    pub fn commit(self) -> FieldElement {
        self.tree.commit()
    }
}

/// Represents a transaction, for now we define a type here but we'll later use the global tx type
/// which will be easier to handle.
pub struct Transaction {
    /// The transaction hash.
    pub tx_hash: H256,
    /// The signature of the transaction (might be empty).
    pub signature: Vec<H256>,
}

/// Calculate transaction commitment hash value.
///
/// The transaction commitment is the root of the Patricia Merkle tree with height 64
/// constructed by adding the (transaction_index, transaction_hash_with_signature)
/// key-value pairs to the tree and computing the root hash.
pub fn calculate_transaction_commitment(transactions: &[Transaction]) -> FieldElement {
    let mut tree = CommitmentTree::default();

    transactions.iter().enumerate().for_each(|(idx, tx)| {
        let idx: u64 = idx.try_into().expect("too many transactions while calculating commitment");
        let final_hash = calculate_transaction_hash_with_signature(tx);
        tree.set(idx, final_hash);
    });
    tree.commit()
}

/// Compute the combined hash of the transaction hash and the signature.
///
/// Since the transaction hash doesn't take the signature values as its input
/// computing the transaction commitent uses a hash value that combines
/// the transaction hash with the array of signature values.
///
/// Note that for non-invoke transactions we don't actually have signatures. The
/// cairo-lang uses an empty list (whose hash is not the ZERO value!) in that
/// case.
fn calculate_transaction_hash_with_signature(tx: &Transaction) -> FieldElement {
    lazy_static::lazy_static!(
        static ref HASH_OF_EMPTY_LIST: FieldElement = HashChain::default().finalize();
    );

    let signature_hash = if tx.signature.is_empty() {
        *HASH_OF_EMPTY_LIST
    } else {
        let mut hash = HashChain::default();
        for signature in &tx.signature {
            hash.update(FieldElement::from_byte_slice_be(signature.as_bytes()).unwrap());
        }
        hash.finalize()
    };

    pedersen::PedersenHasher::hash(FieldElement::from_byte_slice_be(tx.tx_hash.as_bytes()).unwrap(), signature_hash)
}

/// HashChain is the structure used over at cairo side to represent the hash construction needed
/// for computing the class hash.
///
/// Empty hash chained value equals `H(0, 0)` where `H` is the [`stark_hash()`] function, and the
/// second value is the number of values hashed together in this chain. For other values, the
/// accumulator is on each update replaced with the `H(hash, value)` and the number of count
/// incremented by one.
#[derive(Default)]
pub struct HashChain {
    hash: FieldElement,
    count: usize,
}

impl HashChain {
    /// Copy pasted from pathfinder ;).
    pub fn update(&mut self, value: FieldElement) {
        self.hash = pedersen::PedersenHasher::hash(self.hash, value);
        self.count = self.count.checked_add(1).expect("could not have deserialized larger than usize Vecs");
    }
    /// Copy pasted from pathfinder ;).
    pub fn finalize(self) -> FieldElement {
        let count =
            FieldElement::from_byte_slice_be(&self.count.to_be_bytes()).expect("usize is smaller than 251-bits");
        pedersen::PedersenHasher::hash(self.hash, count)
    }
}
