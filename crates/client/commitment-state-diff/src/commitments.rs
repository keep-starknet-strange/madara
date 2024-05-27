use std::collections::HashSet;

use blockifier::state::cached_state::CommitmentStateDiff;
use indexmap::IndexMap;
use mc_db::storage_handler::{self, DeoxysStorageError, StorageView};
use mp_convert::field_element::FromFieldElement;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::poseidon::PoseidonHasher;
use mp_hashers::HasherT;
use rayon::prelude::*;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Event, Transaction};
use starknet_core::types::{
    ContractStorageDiffItem, DeclaredClassItem, DeployedContractItem, NonceUpdate, ReplacedClassItem, StateUpdate,
    StorageEntry,
};
use starknet_ff::FieldElement;
use starknet_types_core::felt::Felt;

use super::events::memory_event_commitment;
use super::transactions::memory_transaction_commitment;

/// Calculate the transaction and event commitment.
///
/// # Arguments
///
/// * `transactions` - The transactions of the block
/// * `events` - The events of the block
/// * `chain_id` - The current chain id
/// * `block_number` - The current block number
///
/// # Returns
///
/// The transaction and the event commitment as `Felt252Wrapper`.
pub fn calculate_commitments(
    transactions: &[Transaction],
    events: &[Event],
    chain_id: Felt252Wrapper,
    block_number: u64,
) -> (Felt252Wrapper, Felt252Wrapper) {
    let (commitment_tx, commitment_event) = rayon::join(
        || memory_transaction_commitment(transactions, chain_id, block_number),
        || memory_event_commitment(events),
    );
    (
        commitment_tx.expect("Failed to calculate transaction commitment"),
        commitment_event.expect("Failed to calculate event commitment"),
    )
}

/// Aggregates all the changes from last state update in a way that is easy to access
/// when computing the state root
///
/// * `state_update`: The last state update fetched from the sequencer
pub fn build_commitment_state_diff(state_update: &StateUpdate) -> CommitmentStateDiff {
    let mut commitment_state_diff = CommitmentStateDiff {
        address_to_class_hash: IndexMap::new(),
        address_to_nonce: IndexMap::new(),
        storage_updates: IndexMap::new(),
        class_hash_to_compiled_class_hash: IndexMap::new(),
    };

    for DeployedContractItem { address, class_hash } in state_update.state_diff.deployed_contracts.iter() {
        let address = ContractAddress::from_field_element(address);
        let class_hash = if address == ContractAddress::from_field_element(FieldElement::ZERO) {
            // System contracts doesnt have class hashes
            ClassHash::from_field_element(FieldElement::ZERO)
        } else {
            ClassHash::from_field_element(class_hash)
        };
        commitment_state_diff.address_to_class_hash.insert(address, class_hash);
    }

    for ReplacedClassItem { contract_address, class_hash } in state_update.state_diff.replaced_classes.iter() {
        let address = ContractAddress::from_field_element(contract_address);
        let class_hash = ClassHash::from_field_element(class_hash);
        commitment_state_diff.address_to_class_hash.insert(address, class_hash);
    }

    for DeclaredClassItem { class_hash, compiled_class_hash } in state_update.state_diff.declared_classes.iter() {
        let class_hash = ClassHash::from_field_element(class_hash);
        let compiled_class_hash = CompiledClassHash::from_field_element(compiled_class_hash);
        commitment_state_diff.class_hash_to_compiled_class_hash.insert(class_hash, compiled_class_hash);
    }

    for NonceUpdate { contract_address, nonce } in state_update.state_diff.nonces.iter() {
        let contract_address = ContractAddress::from_field_element(contract_address);
        let nonce_value = Nonce::from_field_element(nonce);
        commitment_state_diff.address_to_nonce.insert(contract_address, nonce_value);
    }

    for ContractStorageDiffItem { address, storage_entries } in state_update.state_diff.storage_diffs.iter() {
        let contract_address = ContractAddress::from_field_element(address);
        let mut storage_map = IndexMap::new();
        for StorageEntry { key, value } in storage_entries.iter() {
            let key = StorageKey::from_field_element(key);
            let value = StarkFelt::from_field_element(value);
            storage_map.insert(key, value);
        }
        commitment_state_diff.storage_updates.insert(contract_address, storage_map);
    }

    commitment_state_diff
}

/// Calculate state commitment hash value.
///
/// The state commitment is the digest that uniquely (up to hash collisions) encodes the state.
/// It combines the roots of two binary Merkle-Patricia tries of height 251 using Poseidon/Pedersen
/// hashers.
///
/// # Arguments
///
/// * `contracts_trie_root` - The root of the contracts trie.
/// * `classes_trie_root` - The root of the classes trie.
///
/// # Returns
///
/// The state commitment as a `Felt252Wrapper`.
pub fn calculate_state_root<H: HasherT>(
    contracts_trie_root: Felt252Wrapper,
    classes_trie_root: Felt252Wrapper,
) -> Felt252Wrapper
where
    H: HasherT,
{
    let starknet_state_prefix = Felt252Wrapper::try_from("STARKNET_STATE_V0".as_bytes()).unwrap();

    if classes_trie_root == Felt252Wrapper::ZERO {
        contracts_trie_root
    } else {
        let state_commitment_hash =
            H::compute_hash_on_elements(&[starknet_state_prefix.0, contracts_trie_root.0, classes_trie_root.0]);

        state_commitment_hash.into()
    }
}

/// Update the state commitment hash value.
///
/// The state commitment is the digest that uniquely (up to hash collisions) encodes the state.
/// It combines the roots of two binary Merkle-Patricia tries of height 251 using Poseidon/Pedersen
/// hashers.
///
/// # Arguments
///
/// * `CommitmentStateDiff` - The commitment state diff inducing unprocessed state changes.
/// * `BonsaiDb` - The database responsible for storing computing the state tries.
///
///
/// The updated state root as a `Felt252Wrapper`.
pub fn update_state_root(csd: CommitmentStateDiff, block_number: u64) -> Felt252Wrapper {
    // Update contract and its storage tries
    let (contract_trie_root, class_trie_root) = rayon::join(
        || contract_trie_root(&csd, block_number).expect("Failed to compute contract root"),
        || class_trie_root(&csd, block_number).expect("Failed to compute class root"),
    );
    calculate_state_root::<PoseidonHasher>(contract_trie_root, class_trie_root)
}

/// Calculates the contract trie root
///
/// # Arguments
///
/// * `csd`             - Commitment state diff for the current block.
/// * `bonsai_contract` - Bonsai db used to store contract hashes.
/// * `block_number`    - The current block number.
///
/// # Returns
///
/// The contract root.
fn contract_trie_root(csd: &CommitmentStateDiff, block_number: u64) -> Result<Felt252Wrapper, DeoxysStorageError> {
    // NOTE: handlers implicitely acquire a lock on their respective tries
    // for the duration of their livetimes
    let mut handler_contract = storage_handler::contract_trie_mut();
    let mut handler_storage_trie = storage_handler::contract_storage_trie_mut();

    // First we insert the contract storage changes
    for (contract_address, updates) in csd.storage_updates.iter() {
        handler_storage_trie.init(contract_address)?;

        for (key, value) in updates {
            handler_storage_trie.insert(*contract_address, *key, *value)?;
        }
    }

    // Then we commit them
    handler_storage_trie.commit(block_number)?;

    // We need to initialize the contract trie for each contract that has a class_hash or nonce update
    // to retrieve the corresponding storage root
    for contract_address in csd.address_to_class_hash.keys().chain(csd.address_to_nonce.keys()) {
        if !csd.storage_updates.contains_key(contract_address) {
            // Initialize the storage trie if this contract address does not have storage updates
            handler_storage_trie.init(contract_address)?;
        }
    }

    // We need to calculate the contract_state_leaf_hash for each contract
    // that not appear in the storage_updates but has a class_hash or nonce update
    let all_contract_address: HashSet<ContractAddress> = csd
        .storage_updates
        .keys()
        .chain(csd.address_to_class_hash.keys())
        .chain(csd.address_to_nonce.keys())
        .cloned()
        .collect();

    // Then we compute the leaf hashes retrieving the corresponding storage root
    let updates = all_contract_address
        .iter()
        .par_bridge()
        .map(|contract_address| {
            let storage_root = handler_storage_trie.root(contract_address)?;
            let leaf_hash = contract_state_leaf_hash(csd, contract_address, storage_root)?;

            Ok::<(&ContractAddress, Felt), DeoxysStorageError>((contract_address, leaf_hash))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // then we compute the contract root by applying the changes so far
    handler_contract.update(updates)?;
    handler_contract.commit(block_number)?;

    Ok(handler_contract.root()?.into())
}

fn contract_state_leaf_hash(
    csd: &CommitmentStateDiff,
    contract_address: &ContractAddress,
    storage_root: Felt,
) -> Result<Felt, DeoxysStorageError> {
    let (class_hash, nonce) = class_hash_and_nonce(csd, contract_address)?;

    let storage_root = FieldElement::from_bytes_be(&storage_root.to_bytes_be()).unwrap();

    // computes the contract state leaf hash
    let contract_state_hash = PedersenHasher::hash_elements(class_hash, storage_root);
    let contract_state_hash = PedersenHasher::hash_elements(contract_state_hash, nonce);
    let contract_state_hash = PedersenHasher::hash_elements(contract_state_hash, FieldElement::ZERO);

    Ok(Felt::from_bytes_be(&contract_state_hash.to_bytes_be()))
}

fn class_hash_and_nonce(
    csd: &CommitmentStateDiff,
    contract_address: &ContractAddress,
) -> Result<(FieldElement, FieldElement), DeoxysStorageError> {
    let class_hash = csd.address_to_class_hash.get(contract_address);
    let nonce = csd.address_to_nonce.get(contract_address);

    let (class_hash, nonce) = match (class_hash, nonce) {
        (Some(class_hash), Some(nonce)) => (*class_hash, *nonce),
        (Some(class_hash), None) => {
            let nonce = storage_handler::contract_data().get_nonce(contract_address)?.unwrap_or_default();
            (*class_hash, nonce)
        }
        (None, Some(nonce)) => {
            let class_hash = storage_handler::contract_data().get_class_hash(contract_address)?.unwrap_or_default();
            (class_hash, *nonce)
        }
        (None, None) => {
            let contract_data = storage_handler::contract_data().get(contract_address)?.unwrap_or_default();
            let nonce = contract_data.nonce.get().cloned().unwrap_or_default();
            let class_hash = contract_data.class_hash.get().cloned().unwrap_or_default();
            (class_hash, nonce)
        }
    };
    Ok((FieldElement::from_bytes_be(&class_hash.0.0).unwrap(), FieldElement::from_bytes_be(&nonce.0.0).unwrap()))
}

// "CONTRACT_CLASS_LEAF_V0"
const CONTRACT_CLASS_HASH_VERSION: FieldElement =
    FieldElement::from_mont([9331882290187415277, 12057587991035439952, 18444375821049509847, 115292049744600508]);

/// Calculates the class trie root
///
/// # Arguments
///
/// * `csd`          - Commitment state diff for the current block.
/// * `bonsai_class` - Bonsai db used to store class hashes.
/// * `block_number` - The current block number.
///
/// # Returns
///
/// The class root.
fn class_trie_root(csd: &CommitmentStateDiff, block_number: u64) -> Result<Felt252Wrapper, DeoxysStorageError> {
    let mut handler_class = storage_handler::class_trie_mut();

    let updates = csd
        .class_hash_to_compiled_class_hash
        .iter()
        .par_bridge()
        .map(|(class_hash, compiled_class_hash)| {
            let compiled_class_hash = FieldElement::from_bytes_be(&compiled_class_hash.0.0).unwrap();

            let hash = PoseidonHasher::hash_elements(CONTRACT_CLASS_HASH_VERSION, compiled_class_hash);

            (class_hash, hash)
        })
        .collect::<Vec<_>>();

    handler_class.init()?;
    handler_class.update(updates)?;
    handler_class.commit(block_number)?;

    Ok(handler_class.root()?.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_class_hash_version() {
        assert_eq!(
            CONTRACT_CLASS_HASH_VERSION,
            FieldElement::from_byte_slice_be("CONTRACT_CLASS_LEAF_V0".as_bytes()).unwrap(),
        );
    }
}
