use bitvec::vec::BitVec;
use bonsai_trie::databases::HashMapDb;
use bonsai_trie::id::{BasicId, BasicIdBuilder};
use bonsai_trie::{BonsaiStorage, BonsaiStorageConfig};
use mc_db::storage_handler::bonsai_identifier;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use rayon::prelude::*;
use starknet_api::transaction::Event;
use starknet_ff::FieldElement;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Pedersen;

/// Calculate the hash of the event.
///
/// # Arguments
///
/// * `event` - The event we want to calculate the hash of.
///
/// # Returns
///
/// The event hash as `FieldElement`.
pub fn calculate_event_hash<H: HasherT>(event: &Event) -> FieldElement {
    let (keys_hash, data_hash) = rayon::join(
        || {
            H::compute_hash_on_elements(
                &event
                    .content
                    .keys
                    .iter()
                    .map(|key| FieldElement::from(Felt252Wrapper::from(key.0)))
                    .collect::<Vec<FieldElement>>(),
            )
        },
        || {
            H::compute_hash_on_elements(
                &event
                    .content
                    .data
                    .0
                    .iter()
                    .map(|data| FieldElement::from(Felt252Wrapper::from(*data)))
                    .collect::<Vec<FieldElement>>(),
            )
        },
    );
    let from_address = FieldElement::from(Felt252Wrapper::from(event.from_address.0.0));
    H::compute_hash_on_elements(&[from_address, keys_hash, data_hash])
}

/// Calculate the event commitment in memory using HashMapDb (which is more efficient for this
/// usecase).
///
/// # Arguments
///
/// * `events` - The events of the block
///
/// # Returns
///
/// The event commitment as `Felt252Wrapper`.
pub fn memory_event_commitment(events: &[Event]) -> Result<Felt252Wrapper, String> {
    // TODO @cchudant refacto/optimise this function
    if events.is_empty() {
        return Ok(Felt252Wrapper::ZERO);
    }

    let config = BonsaiStorageConfig::default();
    let bonsai_db = HashMapDb::<BasicId>::default();
    let mut bonsai_storage =
        BonsaiStorage::<_, _, Pedersen>::new(bonsai_db, config).expect("Failed to create bonsai storage");
    let identifier = bonsai_identifier::EVENT;

    // event hashes are computed in parallel
    let events = events.par_iter().map(calculate_event_hash::<PedersenHasher>).collect::<Vec<_>>();

    // once event hashes have finished computing, they are inserted into the local Bonsai db
    for (i, event_hash) in events.into_iter().enumerate() {
        let key = BitVec::from_vec(i.to_be_bytes().to_vec());
        let value = Felt::from(Felt252Wrapper::from(event_hash));
        bonsai_storage.insert(identifier, key.as_bitslice(), &value).expect("Failed to insert into bonsai storage");
    }

    // Note that committing changes still has the greatest performance hit
    // as this is where the root hash is calculated. Due to the Merkle structure
    // of Bonsai Tries, this results in a trie size that grows very rapidly with
    // each new insertion. It seems that the only vector of optimization here
    // would be to optimize the tree traversal and hash computation.
    let mut id_builder = BasicIdBuilder::new();
    let id = id_builder.new_id();

    // run in a blocking-safe thread to avoid starving the thread pool
    bonsai_storage.commit(id).expect("Failed to commit to bonsai storage");
    let root_hash = bonsai_storage.root_hash(identifier).expect("Failed to get root hash");

    Ok(Felt252Wrapper::from(root_hash))
}
