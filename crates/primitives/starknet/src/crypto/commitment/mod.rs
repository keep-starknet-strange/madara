use alloc::vec;
use alloc::vec::Vec;

use bitvec::prelude::Msb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use starknet_crypto::FieldElement;

use super::hash::pedersen::PedersenHasher;
use super::merkle_patricia_tree::merkle_tree::{MerkleTree, ProofNode};
use super::merkle_patricia_tree::ref_merkle_tree::RefMerkleTree;
use crate::execution::types::Felt252Wrapper;
use crate::traits::hash::CryptoHasherT;
use crate::transaction::types::{
    DeclareTransaction, DeployAccountTransaction, EventWrapper, InvokeTransaction, Transaction,
};

/// Hash of the leaf of the ClassCommitment tree
pub type ClassCommitmentLeafHash = Felt252Wrapper;

/// A Patricia Merkle tree with height 64 used to compute transaction and event commitments.
///
/// According to the [documentation](https://docs.starknet.io/docs/Blocks/header/#block-header)
/// the commitment trees are of height 64, because the key used is the 64 bit representation
/// of the index of the transaction / event within the block.
///
/// The tree height is 64 in our case since our set operation takes u64 index values.
struct CommitmentTree<T: CryptoHasherT> {
    tree: RefMerkleTree<T>,
}

impl<T: CryptoHasherT> Default for CommitmentTree<T> {
    fn default() -> Self {
        Self { tree: RefMerkleTree::empty() }
    }
}

impl<T: CryptoHasherT> CommitmentTree<T> {
    /// Sets the value of a key in the merkle tree.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to set.
    /// * `value` - The value to set.
    pub fn set(&mut self, index: u64, value: FieldElement) {
        let key = index.to_be_bytes();
        self.tree.set(&BitVec::from_vec(key.to_vec()), Felt252Wrapper(value))
    }

    /// Get the merkle root of the tree.
    pub fn commit(self) -> Felt252Wrapper {
        self.tree.commit()
    }
}

/// A Patricia Merkle tree with height 251 used to compute transaction and event commitments.
///
/// According to the [documentation](https://docs.starknet.io/docs/Blocks/header/#block-header)
/// the commitment trees are of height 251, because the key used is a Field Element.
///
/// The tree height is 251 in our case since our set operation takes Fieldelement index values.
#[derive(Clone, Debug, PartialEq, scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)]
pub struct StateCommitmentTree<T: CryptoHasherT> {
    tree: MerkleTree<T>,
}

impl<T: CryptoHasherT> Default for StateCommitmentTree<T> {
    fn default() -> Self {
        Self { tree: MerkleTree::empty() }
    }
}

impl<T: CryptoHasherT> StateCommitmentTree<T> {
    /// Sets the value of a key in the merkle tree.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to set.
    /// * `value` - The value to set.
    pub fn set(&mut self, index: Felt252Wrapper, value: Felt252Wrapper) {
        let key = &index.0.to_bytes_be()[..31];
        self.tree.set(&BitVec::from_vec(key.to_vec()), value)
    }

    /// Get the merkle root of the tree.
    pub fn commit(&mut self) -> Felt252Wrapper {
        self.tree.commit()
    }

    #[allow(dead_code)]
    /// Generates a proof for `key`. See [`MerkleTree::get_proof`].
    pub fn get_proof(&self, key: &BitSlice<u8, Msb0>) -> Vec<ProofNode> {
        self.tree.get_proof(key)
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
pub fn calculate_commitments<T: CryptoHasherT>(
    transactions: &[Transaction],
    events: &[EventWrapper],
) -> (Felt252Wrapper, Felt252Wrapper) {
    (calculate_transaction_commitment::<T>(transactions), calculate_event_commitment::<T>(events))
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
pub fn calculate_transaction_commitment<T: CryptoHasherT>(transactions: &[Transaction]) -> Felt252Wrapper {
    let mut tree = CommitmentTree::<T>::default();

    transactions.iter().enumerate().for_each(|(idx, tx)| {
        let idx: u64 = idx.try_into().expect("too many transactions while calculating commitment");
        let final_hash = calculate_transaction_hash_with_signature::<T>(tx);
        tree.set(idx, final_hash);
    });
    tree.commit()
}

/// Calculate transaction commitment hash value.
///
/// The event commitment is the root of the Patricia Merkle tree with height 64
/// constructed by adding the event hash
/// (see https://docs.starknet.io/documentation/architecture_and_concepts/Events/starknet-events/#event_hash)
/// to the tree and computing the root hash.
///
/// # Arguments
///
/// * `transactions` - The transactions to get the events from.
///
/// # Returns
///
/// The merkle root of the merkle tree built from the transactions and the number of events.
pub fn calculate_event_commitment<T: CryptoHasherT>(events: &[EventWrapper]) -> Felt252Wrapper {
    let mut tree = CommitmentTree::<T>::default();
    events.iter().enumerate().for_each(|(id, event)| {
        let final_hash = calculate_event_hash::<T>(event);
        tree.set(id as u64, final_hash);
    });
    tree.commit()
}

/// Calculate class commitment tree leaf hash value.
///
/// See: <https://docs.starknet.io/documentation/architecture_and_concepts/State/starknet-state/#classes_tree>
///
/// # Arguments
///
/// * `compiled_class_hash` - The hash of the compiled class.
///
/// # Returns
///
/// The hash of the class commitment tree leaf.
pub fn calculate_class_commitment_leaf_hash<T: CryptoHasherT>(
    compiled_class_hash: Felt252Wrapper,
) -> ClassCommitmentLeafHash {
    let contract_class_hash_version = Felt252Wrapper::try_from("CONTRACT_CLASS_LEAF_V0".as_bytes()).unwrap(); // Unwrap safu

    let hash = <T>::compute_hash_on_elements(&[contract_class_hash_version.0, compiled_class_hash.0]);

    hash.into()
}

/// Calculate class commitment tree root hash value.
///
/// The classes tree encodes the information about the existing classes in the state of Starknet.
/// It maps (Cairo 1.0) class hashes to their compiled class hashes
///
/// # Arguments
///
/// * `classes` - The classes to get the root from.
///
/// # Returns
///
/// The merkle root of the merkle tree built from the classes.
pub fn calculate_class_commitment_tree_root_hash<T: CryptoHasherT>(class_hashes: &[Felt252Wrapper]) -> Felt252Wrapper {
    let mut tree = StateCommitmentTree::<T>::default();
    class_hashes.iter().for_each(|class_hash| {
        let final_hash = calculate_class_commitment_leaf_hash::<T>(*class_hash);
        tree.set(*class_hash, final_hash);
    });
    tree.commit()
}

/// Calculates the contract state hash from its preimage.
///
/// # Arguments
///
/// * `hash` - The hash of the contract definition.
/// * `root` - The root of root of another Merkle-Patricia tree of height 251 that is constructed
///   from the contract’s storage.
/// * `nonce` - The current nonce of the contract.
///
/// # Returns
///
/// The contract state hash.
pub fn calculate_contract_state_hash<T: CryptoHasherT>(
    hash: Felt252Wrapper,
    root: Felt252Wrapper,
    nonce: Felt252Wrapper,
) -> Felt252Wrapper {
    const CONTRACT_STATE_HASH_VERSION: Felt252Wrapper = Felt252Wrapper::ZERO;

    // The contract state hash is defined as H(H(H(hash, root), nonce), CONTRACT_STATE_HASH_VERSION)
    let hash = <T>::compute_hash_on_elements(&[hash.0, root.0, nonce.0, CONTRACT_STATE_HASH_VERSION.0]);

    // Compare this with the HashChain construction used in the contract_hash: the number of
    // elements is not hashed to this hash, and this is supposed to be different.
    hash.into()
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
    T: CryptoHasherT,
{
    let signature_hash = <T as CryptoHasherT>::compute_hash_on_elements(
        &tx.signature.iter().map(|elt| FieldElement::from(*elt)).collect::<Vec<FieldElement>>(),
    );
    <T as CryptoHasherT>::hash(FieldElement::from(tx.hash), signature_hash)
}
/// Computes the transaction hash of an invoke transaction.
///
/// # Argument
///
/// * `transaction` - The invoke transaction to get the hash of.
pub fn calculate_invoke_tx_hash(transaction: InvokeTransaction, chain_id: Felt252Wrapper) -> Felt252Wrapper {
    calculate_transaction_hash_common::<PedersenHasher>(
        transaction.sender_address,
        transaction.calldata.as_slice(),
        transaction.max_fee,
        transaction.nonce,
        transaction.version,
        b"invoke",
        chain_id,
    )
}

/// Computes the transaction hash of a declare transaction.
///
/// # Argument
///
/// * `transaction` - The declare transaction to get the hash of.
pub fn calculate_declare_tx_hash(transaction: DeclareTransaction, chain_id: Felt252Wrapper) -> Felt252Wrapper {
    calculate_transaction_hash_common::<PedersenHasher>(
        transaction.sender_address,
        &[transaction.compiled_class_hash],
        transaction.max_fee,
        transaction.nonce,
        transaction.version,
        b"declare",
        chain_id,
    )
}

/// Computes the transaction hash of a deploy account transaction.
///
/// # Argument
///
/// * `transaction` - The deploy account transaction to get the hash of.
pub fn calculate_deploy_account_tx_hash(
    transaction: DeployAccountTransaction,
    chain_id: Felt252Wrapper,
    address: Felt252Wrapper,
) -> Felt252Wrapper {
    calculate_transaction_hash_common::<PedersenHasher>(
        address,
        &vec![vec![transaction.account_class_hash, transaction.salt], transaction.calldata.to_vec()].concat(),
        transaction.max_fee,
        transaction.nonce,
        transaction.version,
        b"deploy_account",
        chain_id,
    )
}

/// Computes the transaction hash using a hash function of type T
pub fn calculate_transaction_hash_common<T>(
    sender_address: Felt252Wrapper,
    calldata: &[Felt252Wrapper],
    max_fee: Felt252Wrapper,
    nonce: Felt252Wrapper,
    version: u8,
    tx_prefix: &[u8],
    chain_id: Felt252Wrapper,
) -> Felt252Wrapper
where
    T: CryptoHasherT,
{
    // All the values are validated before going through this function so it's safe to unwrap.
    let sender_address = FieldElement::from_bytes_be(&sender_address.into()).unwrap();
    let calldata_hash = <T as CryptoHasherT>::compute_hash_on_elements(
        &calldata.iter().map(|&val| FieldElement::from(val)).collect::<Vec<FieldElement>>(),
    );
    let max_fee = FieldElement::from_bytes_be(&max_fee.into()).unwrap();
    let nonce = FieldElement::from_bytes_be(&nonce.into()).unwrap();
    let version = FieldElement::from_byte_slice_be(&version.to_be_bytes()).unwrap();
    let tx_prefix = FieldElement::from_byte_slice_be(tx_prefix).unwrap();

    let tx_hash = <T as CryptoHasherT>::compute_hash_on_elements(&vec![
        tx_prefix,
        version,
        sender_address,
        FieldElement::ZERO,
        calldata_hash,
        max_fee,
        chain_id.0,
        nonce,
    ]);

    tx_hash.into()
}

/// Calculate the hash of an event.
///
/// See the [documentation](https://docs.starknet.io/docs/Events/starknet-events#event-hash)
/// for details.
pub fn calculate_event_hash<T: CryptoHasherT>(event: &EventWrapper) -> FieldElement {
    let keys_hash = T::compute_hash_on_elements(
        &event.keys.iter().map(|key| FieldElement::from(*key)).collect::<Vec<FieldElement>>(),
    );
    let data_hash = T::compute_hash_on_elements(
        &event.data.iter().map(|data| FieldElement::from(*data)).collect::<Vec<FieldElement>>(),
    );
    let from_address = FieldElement::from(event.from_address);
    T::compute_hash_on_elements(&[from_address, keys_hash, data_hash])
}
