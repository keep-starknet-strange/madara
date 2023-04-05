use alloc::vec::Vec;

use bitvec::vec::BitVec;
use sp_core::hexdisplay::AsBytesRef;
use sp_core::H256;
use starknet_api::api_core::ClassHash;
use starknet_api::hash::StarkFelt;
use starknet_crypto::FieldElement;

use super::merkle_patricia_tree::merkle_tree::MerkleTree;
use crate::state::ContractStateNode;
use crate::traits::hash::CryptoHasher;
use crate::transaction::types::{EventWrapper, Transaction};

pub const STARKNET_STATE_V0: &[u8] = b"STARKNET_STATE_V0";
pub const CONTRACT_CLASS_LEAF_V0: &[u8] = b"CONTRACT_CLASS_LEAF_V0";

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
}

/// Calculate the transaction commitment, the event commitment and the event count.
///
/// # Arguments
///
/// * `transactions` - The transactions of the block
///
/// # Returns
///
/// The transaction commitment, the event commitment and the event count.
pub fn calculate_commitments<T: CryptoHasher>(transactions: &[Transaction]) -> (H256, (H256, u128)) {
    (calculate_transaction_commitment::<T>(transactions), calculate_event_commitment::<T>(transactions))
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
pub fn calculate_transaction_commitment<T: CryptoHasher>(transactions: &[Transaction]) -> H256 {
    let mut tree = CommitmentTree::<T>::default();

    transactions.iter().enumerate().for_each(|(idx, tx)| {
        let idx: u64 = idx.try_into().expect("too many transactions while calculating commitment");
        let final_hash = calculate_transaction_hash_with_signature::<T>(tx);
        tree.set(idx, final_hash);
    });
    H256::from_slice(&tree.commit().to_bytes_be())
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
pub fn calculate_event_commitment<T: CryptoHasher>(transactions: &[Transaction]) -> (H256, u128) {
    let mut tree = CommitmentTree::<T>::default();
    let mut len = 0_u64;
    transactions.iter().flat_map(|tx| tx.events.iter()).for_each(|event| {
        len += 1;
        let final_hash = calculate_event_hash::<T>(event);
        tree.set(len - 1, final_hash);
    });
    (H256::from_slice(&tree.commit().to_bytes_be()), len as u128)
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
fn calculate_transaction_hash_with_signature<T>(tx: &Transaction) -> FieldElement
where
    T: CryptoHasher,
{
    let signature_hash = <T as CryptoHasher>::compute_hash_on_elements(
        &tx.signature
            .iter()
            .map(|elt| FieldElement::from_byte_slice_be(elt.as_bytes()).unwrap())
            .collect::<Vec<FieldElement>>(),
    );
    <T as CryptoHasher>::hash(FieldElement::from_byte_slice_be(tx.hash.as_bytes()).unwrap(), signature_hash)
}

/// Calculate the hash of an event.
///
/// See the [documentation](https://docs.starknet.io/docs/Events/starknet-events#event-hash)
/// for details.
pub fn calculate_event_hash<T: CryptoHasher>(event: &EventWrapper) -> FieldElement {
    let keys_hash = T::compute_hash_on_elements(
        &event
            .keys
            .iter()
            .map(|key| FieldElement::from_byte_slice_be(key.as_bytes()).unwrap())
            .collect::<Vec<FieldElement>>(),
    );
    let data_hash = T::compute_hash_on_elements(
        &event
            .data
            .iter()
            .map(|data| FieldElement::from_byte_slice_be(data.as_bytes()).unwrap())
            .collect::<Vec<FieldElement>>(),
    );
    let from_address = FieldElement::from_byte_slice_be(event.from_address.as_bytes_ref()).unwrap();
    T::compute_hash_on_elements(&[from_address, keys_hash, data_hash])
}

/// Calculate the contracts tree root.
pub fn calculate_contract_state_root<T: CryptoHasher>(contracts: &[ContractStateNode]) -> H256 {
    let mut tree = CommitmentTree::<T>::default();
    contracts.iter().enumerate().for_each(|(idx, contract)| {
        let idx: u64 = idx.try_into().expect("too many contracts while calculating commitment");
        let final_hash = calculate_contract_state_node_hash::<T>(contract);
        tree.set(idx, final_hash);
    });
    H256::from_slice(&tree.commit().to_bytes_be())
}

/// Calculate the hash of a contract state node.
fn calculate_contract_state_node_hash<T: CryptoHasher>(contract: &ContractStateNode) -> FieldElement {
    let class_hash = FieldElement::from_byte_slice_be(contract.class_hash.0.bytes()).unwrap();
    let storage_root = FieldElement::from_byte_slice_be(contract.storage_root.bytes()).unwrap();
    let nonce = FieldElement::from_byte_slice_be(contract.nonce.0.bytes()).unwrap();
    let hash_l1 = T::compute_hash_on_elements(&[class_hash, storage_root]);
    let hash_l2 = T::compute_hash_on_elements(&[hash_l1, nonce]);
    return T::compute_hash_on_elements(&[hash_l2, FieldElement::ZERO]);
}

/// Calculate the classes tree root.
pub fn calculate_classes_tree_root<T: CryptoHasher>(classes: &[ClassHash]) -> H256 {
    let mut tree = CommitmentTree::<T>::default();
    classes.iter().enumerate().for_each(|(idx, class)| {
        let idx: u64 = idx.try_into().expect("too many classes while calculating commitment");
        let final_hash = calculate_contract_class_hash::<T>(class);
        tree.set(idx, final_hash);
    });
    H256::from_slice(&tree.commit().to_bytes_be())
}

/// Calculate the hash of a contract class.
fn calculate_contract_class_hash<T: CryptoHasher>(class: &ClassHash) -> FieldElement {
    let class_hash = FieldElement::from_byte_slice_be(class.0.bytes()).unwrap();
    T::compute_hash_on_elements(&[FieldElement::from_byte_slice_be(CONTRACT_CLASS_LEAF_V0).unwrap(), class_hash])
}

/// Calculate the global state root.
pub fn calculate_global_state_root<T: CryptoHasher>(contract_state_root: H256, classes_root: H256) -> H256 {
    let contract_state_root = FieldElement::from_byte_slice_be(contract_state_root.as_bytes()).unwrap();
    let classes_root = FieldElement::from_byte_slice_be(classes_root.as_bytes()).unwrap();
    let root = T::compute_hash_on_elements(&[
        FieldElement::from_byte_slice_be(STARKNET_STATE_V0).unwrap(),
        contract_state_root,
        classes_root,
    ]);
    H256::from_slice(&root.to_bytes_be())
}
