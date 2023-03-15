use alloc::vec::Vec;

use bitvec::vec::BitVec;
use sp_core::H256;
use starknet_crypto::FieldElement;

use super::merkle_patricia_tree::merkle_tree::MerkleTree;
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::traits::hash::CryptoHasher;
use crate::transaction::Transaction;

/// A Patricia Merkle tree with height 64 used to compute transaction and event commitments.
///
/// According to the [documentation](https://docs.starknet.io/docs/Blocks/header/#block-header)
/// the commitment trees are of height 64, because the key used is the 64 bit representation
/// of the index of the transaction / event within the block.
///
/// The tree height is 64 in our case since our set operation takes u64 index values.
struct CommitmentTree<T: CryptoHasher> {
    tree: MerkleTree<T>,
}

impl<T: CryptoHasher> Default for CommitmentTree<T> {
    fn default() -> Self {
        Self { tree: MerkleTree::empty() }
    }
}

impl<T: CryptoHasher> CommitmentTree<T> {
    /// Sets the value of a key in the merkle tree.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to set.
    /// * `value` - The value to set.
    pub fn set(&mut self, index: u64, value: FieldElement) {
        let key = index.to_be_bytes();
        self.tree.set(&BitVec::from(key.to_vec()), value)
    }

    /// Get the merkle root of the tree.
    pub fn commit(self) -> FieldElement {
        self.tree.commit()
    }

    /// Compute the combined hash of the transaction hash and the signature.
    ///
    /// Since the transaction hash doesn't take the signature values as its input
    /// computing the transaction commitent uses a hash value that combines
    /// the transaction hash with the array of signature values.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction to compute the hash of.
    ///
    /// # Returns
    ///
    /// The transaction hash with signature.
    pub fn calculate_transaction_hash_with_signature(&self, tx: &Transaction) -> FieldElement {
        let signature_hash = self.compute_hash_on_elements(
            &tx.signature
                .iter()
                .map(|elt| FieldElement::from_byte_slice_be(elt.as_bytes()).unwrap())
                .collect::<Vec<FieldElement>>(),
        );
        <T as CryptoHasher>::hash(FieldElement::from_byte_slice_be(tx.hash.as_bytes()).unwrap(), signature_hash)
    }

    /// Compute hash on elements, base on the [python implementation](https://github.com/starkware-libs/cairo-lang/blob/12ca9e91bbdc8a423c63280949c7e34382792067/src/starkware/cairo/common/hash_state.py#L6-L15).
    ///
    /// # Arguments
    ///
    /// * `elements` - The elements to hash.
    ///
    /// # Returns
    ///
    /// h(h(h(h(0, data[0]), data[1]), ...), data[n-1]), n).
    pub fn compute_hash_on_elements(&self, elements: &[FieldElement]) -> FieldElement {
        if elements.is_empty() {
            <T as CryptoHasher>::hash(FieldElement::ZERO, FieldElement::ZERO)
        } else {
            let hash = elements.iter().fold(FieldElement::ZERO, |a, b| <T as CryptoHasher>::hash(a, *b));
            <T as CryptoHasher>::hash(hash, FieldElement::from_byte_slice_be(&elements.len().to_be_bytes()).unwrap())
        }
    }
}

/// Calculate transaction commitment hash value.
///
/// The transaction commitment is the root of the Patricia Merkle tree with height 64
/// constructed by adding the (transaction_index, transaction_hash_with_signature)
/// key-value pairs to the tree and computing the root hash.
///
/// # Arguments
///
/// * `transactions` - The transactions to get the root from.
///
/// # Returns
///
/// The merkle root of the merkle tree built from the transactions.
pub fn calculate_transaction_commitment(transactions: &[Transaction]) -> H256 {
    let mut tree = CommitmentTree::<PedersenHasher>::default();

    transactions.iter().enumerate().for_each(|(idx, tx)| {
        let idx: u64 = idx.try_into().expect("too many transactions while calculating commitment");
        let final_hash = tree.calculate_transaction_hash_with_signature(tx);
        tree.set(idx, final_hash);
    });
    H256::from_slice(&tree.commit().to_bytes_be())
}
