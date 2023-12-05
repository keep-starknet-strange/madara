use std::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CommitmentStateDiff;
use frame_support::{Identity, StorageHasher};
#[cfg(not(feature = "std"))]
use hashbrown::hash_map::DefaultHashBuilder as HasherBuilder;
use indexmap::IndexMap;
use log::info;
use madara_runtime::{Block as SubstrateBlock, Header as SubstrateHeader};
use mc_db::MappingCommitment;
use mc_rpc_core::utils::get_block_by_block_hash;
use mp_block::{Block, Header};
use mp_digest_log::MADARA_ENGINE_ID;
use mp_hashers::pedersen::PedersenHasher;
use mp_storage::{
    SN_COMPILED_CLASS_HASH_PREFIX, SN_CONTRACT_CLASS_HASH_PREFIX, SN_CONTRACT_CLASS_PREFIX, SN_NONCE_PREFIX,
    SN_STORAGE_PREFIX,
};
use sc_client_api::backend::NewBlockState::Best;
use sc_client_api::backend::{Backend, BlockImportOperation};
use sp_blockchain::{HeaderBackend, Info};
use sp_core::{Decode, Encode};
use sp_runtime::generic::{Digest, DigestItem, Header as GenericHeader};
use sp_runtime::traits::BlakeTwo256;
use sp_state_machine::{OverlayedChanges, StorageKey, StorageValue};
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey as StarknetStorageKey;

use super::*;

/// StateWriter is responsible for applying StarkNet state differences to the underlying blockchain
/// state.
pub struct StateWriter<B: BlockT, C, BE> {
    client: Arc<C>,
    substrate_backend: Arc<BE>,
    madara_backend: Arc<mc_db::Backend<B>>,
    phantom_data: PhantomData<B>,
}

impl<B, C, BE> StateWriter<B, C, BE>
where
    B: BlockT<Hash = H256, Header = GenericHeader<u32, BlakeTwo256>>,
    C: HeaderBackend<B>,
    BE: Backend<B>,
{
    /// Creates a new `StateWriter` instance.
    ///
    /// # Arguments
    ///
    /// * `client` - StarkNet runtime client.
    /// * `substrate_backend` - Substrate blockchain backend.
    /// * `madara_backend` - Backend for interacting with the Madara storage.
    pub fn new(client: Arc<C>, substrate_backend: Arc<BE>, madara_backend: Arc<mc_db::Backend<B>>) -> Self {
        Self { client, substrate_backend, madara_backend, phantom_data: PhantomData }
    }

    /// Applies a StarkNet state difference to the underlying blockchain state.
    ///
    /// # Arguments
    ///
    /// * `starknet_block_number` - StarkNet block number associated with the state difference.
    /// * `starknet_block_hash` - StarkNet block hash associated with the state difference.
    /// * `state_diff` - StarkNet state difference to be applied.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the operation fails.
    pub fn apply_state_diff(
        &self,
        starknet_block_number: u64,
        starknet_block_hash: U256,
        state_diff: &StateDiff,
    ) -> Result<(), Error> {
        let mut inner_state_diff = InnerStateDiff::default();

        for (contract_address, class_hash) in state_diff.deployed_contracts.iter() {
            inner_state_diff.commitment.address_to_class_hash.insert(*contract_address, *class_hash);
        }

        for (contract_address, storage_changes) in state_diff.storage_diffs.iter() {
            inner_state_diff.commitment.storage_updates.insert(*contract_address, storage_changes.clone());
        }

        for (class_hash, (compiled_class_hash, _)) in state_diff.declared_classes.iter() {
            inner_state_diff.commitment.class_hash_to_compiled_class_hash.insert(*class_hash, *compiled_class_hash);
        }

        for (contract_address, nonce) in state_diff.nonces.iter() {
            inner_state_diff.commitment.address_to_nonce.insert(*contract_address, *nonce);
        }

        let starknet_block_hash = u256_to_h256(starknet_block_hash);
        self.apply_inner_state_diff(starknet_block_number, starknet_block_hash, inner_state_diff)
    }

    /// Applies the inner StarkNet state difference to the underlying blockchain state.
    ///
    /// This method takes the inner StarkNet state difference, StarkNet block number, and StarkNet
    /// block hash, and applies the changes to the Substrate blockchain state. It calculates the
    /// state root after the storage change and commits the operation to the Substrate backend.
    ///
    /// # Arguments
    ///
    /// * `starknet_block_number` - StarkNet block number associated with the inner state
    ///   difference.
    /// * `starknet_block_hash` - StarkNet block hash associated with the inner state difference.
    /// * `state_diff` - Inner StarkNet state difference to be applied.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the operation fails.
    pub(crate) fn apply_inner_state_diff(
        &self,
        starknet_block_number: u64,
        mut starknet_block_hash: H256,
        state_diff: InnerStateDiff,
    ) -> Result<(), Error> {
        let block_info = self.client.info();

        let starknet_block = self.create_starknet_block(&block_info, starknet_block_number as u32)?;

        // In earlier versions, we couldn't obtain the block hash from L1.
        if starknet_block_hash == H256::default() {
            starknet_block_hash = starknet_block.header().hash::<PedersenHasher>().into();
        }
        let digest = DigestItem::Consensus(MADARA_ENGINE_ID, mp_digest_log::Log::Block(starknet_block).encode());

        let mut substrate_block = SubstrateBlock {
            header: SubstrateHeader {
                parent_hash: block_info.best_hash,
                number: block_info.best_number,
                // todo calculate substrate state root
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest { logs: vec![digest] },
            },
            extrinsics: Default::default(),
        };
        substrate_block.header.number += 1;

        let storage_changes: InnerStorageChangeSet = state_diff.into();
        // TODO use the second element.
        substrate_block.header.state_root =
            self.calculate_state_root_after_storage_change(&storage_changes, block_info.best_hash).0;

        let substrate_block_hash = substrate_block.hash();
        let mut operation = self
            .substrate_backend
            .begin_operation()
            .and_then(|mut op| {
                op.update_storage(storage_changes.changes, storage_changes.child_changes)?;
                op.set_block_data(substrate_block.header, None, None, None, Best)?;
                Ok(op)
            })
            .map_err(|e| Error::ConstructTransaction(e.to_string()))?;

        self.substrate_backend
            .begin_state_operation(&mut operation, block_info.best_hash)
            .map_err(|e| Error::CommitStorage(e.to_string()))?;

        self.substrate_backend.commit_operation(operation).map_err(|e| Error::CommitStorage(e.to_string()))?;

        self.madara_backend
            .mapping()
            .write_hashes(MappingCommitment {
                block_hash: substrate_block_hash,
                starknet_block_hash,
                starknet_transaction_hashes: Vec::new(),
            })
            .map_err(|e| Error::Other(e.to_string()))?;

        info!(target: LOG_TARGET, "~~ apply state diff. starknet block number: {}, starknet block hash: {:#?}", 
            starknet_block_number, starknet_block_hash);
        Ok(())
    }

    /// Creates a StarkNet block based on the best StarkNet block in the blockchain.
    ///
    /// This method takes the information of the best StarkNet block in the blockchain, the desired
    /// block number, and creates a new StarkNet block with the appropriate header.
    ///
    /// # Arguments
    ///
    /// * `block_chain_info` - Information about the best block in the blockchain.
    /// * `block_number` - Desired block number for the new StarkNet block.
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created StarkNet block or an error if the operation fails.
    fn create_starknet_block(&self, block_chain_info: &Info<B>, block_number: u32) -> Result<Block, Error> {
        if block_chain_info.best_number >= block_number {
            return Err(Error::AlreadyInChain);
        }

        let best_starknet_block =
            get_block_by_block_hash(self.client.as_ref(), block_chain_info.best_hash).ok_or(Error::UnknownBlock)?;

        let starknet_header = Header {
            parent_block_hash: best_starknet_block.header().hash::<PedersenHasher>().into(),
            block_number: block_number as u64,
            protocol_version: best_starknet_block.header().protocol_version,
            ..Default::default()
        };

        Ok(Block::new(starknet_header, Default::default()))
    }

    /// Calculates the state root after a storage change.
    ///
    /// This method takes the inner storage change set and the current block hash,
    /// and calculates the new state root after applying the storage changes.
    ///
    /// # Arguments
    ///
    /// * `storage_changes` - Inner storage change set to be applied.
    /// * `block_hash` - Current block hash of the blockchain.
    ///
    /// # Returns
    ///
    /// The calculated state root after the storage change.
    fn calculate_state_root_after_storage_change(
        &self,
        storage_changes: &InnerStorageChangeSet,
        block_hash: H256,
    ) -> (H256, bool) {
        let mut overlay = OverlayedChanges::default();

        // now pallet starknet not use child storages.
        for (k, v) in storage_changes.changes.iter() {
            overlay.set_storage(k.to_vec(), v.clone());
        }
        let trie_backend = self.substrate_backend.state_at(block_hash).unwrap();

        overlay.storage_root(&trie_backend, Default::default())
    }
}

// InnerStorageChangeSet just used for test
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InnerStorageChangeSet {
    pub changes: Vec<(StorageKey, Option<StorageValue>)>,

    #[allow(clippy::type_complexity)]
    pub child_changes: Vec<(StorageKey, Vec<(StorageKey, Option<StorageValue>)>)>,
}

/// InnerStateDiff represents the StarkNet state difference at a more granular level.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct InnerStateDiff {
    pub commitment: CommitmentStateDiff,
    pub declared_classes: IndexMap<ClassHash, ContractClass, HasherBuilder>,
}

impl Default for InnerStateDiff {
    fn default() -> Self {
        InnerStateDiff {
            commitment: CommitmentStateDiff {
                address_to_class_hash: Default::default(),
                address_to_nonce: Default::default(),
                storage_updates: Default::default(),
                class_hash_to_compiled_class_hash: Default::default(),
            },
            declared_classes: Default::default(),
        }
    }
}

impl InnerStorageChangeSet {
    /// Creates an iterator over the changes in the `InnerStorageChangeSet`.
    pub fn iter(&self) -> impl Iterator<Item = (Option<&StorageKey>, &StorageKey, Option<&StorageValue>)> + '_ {
        let top = self.changes.iter().map(|(k, v)| (None, k, v.as_ref()));
        let children = self
            .child_changes
            .iter()
            .flat_map(|(sk, changes)| changes.iter().map(move |(k, v)| (Some(sk), k, v.as_ref())));
        top.chain(children)
    }
}

pub fn storage_key_build(prefix: Vec<u8>, key: &[u8]) -> Vec<u8> {
    [prefix, Identity::hash(key)].concat()
}

/// Converts `InnerStorageChangeSet` to `InnerStateDiff`.
impl From<InnerStorageChangeSet> for InnerStateDiff {
    fn from(value: InnerStorageChangeSet) -> Self {
        let mut state_diff = Self::default();

        for (_prefix, full_storage_key, change) in value.iter() {
            // The storages we are interested in all have prefix of length 32 bytes.
            // The pallet identifier takes 16 bytes, the storage one 16 bytes.
            // So if a storage key is smaller than 32 bytes,
            // the program will panic when we index it to get it's prefix
            if full_storage_key.len() < 32 {
                continue;
            }
            let prefix = &full_storage_key[..32];

            // All the `try_into` are safe to `unwrap` because we know what the storage contains
            // and therefore what size it is
            if prefix == *SN_NONCE_PREFIX {
                let contract_address =
                    ContractAddress(PatriciaKey(StarkFelt(full_storage_key[32..].try_into().unwrap())));
                // `change` is safe to unwrap as `Nonces` storage is `ValueQuery`
                let nonce = Nonce(StarkFelt(change.unwrap().clone().try_into().unwrap()));
                state_diff.commitment.address_to_nonce.insert(contract_address, nonce);
            } else if prefix == *SN_STORAGE_PREFIX {
                let contract_address =
                    ContractAddress(PatriciaKey(StarkFelt(full_storage_key[32..64].try_into().unwrap())));
                let storage_key =
                    StarknetStorageKey(PatriciaKey(StarkFelt(full_storage_key[64..].try_into().unwrap())));
                // `change` is safe to unwrap as `StorageView` storage is `ValueQuery`
                let value = StarkFelt(change.unwrap().clone().try_into().unwrap());

                match state_diff.commitment.storage_updates.get_mut(&contract_address) {
                    Some(contract_storage) => {
                        contract_storage.insert(storage_key, value);
                    }
                    None => {
                        let mut contract_storage: IndexMap<_, _, _> = Default::default();
                        contract_storage.insert(storage_key, value);

                        state_diff.commitment.storage_updates.insert(contract_address, contract_storage);
                    }
                }
            } else if prefix == *SN_CONTRACT_CLASS_HASH_PREFIX {
                let contract_address =
                    ContractAddress(PatriciaKey(StarkFelt(full_storage_key[32..].try_into().unwrap())));
                // `change` is safe to unwrap as `ContractClassHashes` storage is `ValueQuery`
                let class_hash = ClassHash(StarkFelt(change.unwrap().clone().try_into().unwrap()));

                state_diff.commitment.address_to_class_hash.insert(contract_address, class_hash);
            } else if prefix == *SN_COMPILED_CLASS_HASH_PREFIX {
                let class_hash = ClassHash(StarkFelt(full_storage_key[32..].try_into().unwrap()));
                // In the current state of starknet protocol, a compiled class hash can not be erased, so we should
                // never see `change` being `None`. But there have been an "erase contract class" mechanism live on
                // the network during the Regenesis migration. Better safe than sorry.
                let compiled_class_hash = CompiledClassHash(
                    change.map(|data| StarkFelt(data.clone().try_into().unwrap())).unwrap_or_default(),
                );

                state_diff.commitment.class_hash_to_compiled_class_hash.insert(class_hash, compiled_class_hash);
            } else if prefix == *SN_CONTRACT_CLASS_PREFIX {
                let contract_class = change.map(|data| ContractClass::decode(&mut &data[..]).unwrap()).unwrap();
                let class_hash = ClassHash(StarkFelt(full_storage_key[32..].try_into().unwrap()));
                state_diff.declared_classes.insert(class_hash, contract_class);
            }
        }

        state_diff
    }
}

/// Converts `InnerStateDiff` to `InnerStorageChangeSet`.
impl From<InnerStateDiff> for InnerStorageChangeSet {
    fn from(inner_state_diff: InnerStateDiff) -> Self {
        let mut changes: Vec<(StorageKey, Option<StorageValue>)> = Vec::new();
        // now starknet not use child changes.
        let mut _child_changes = Vec::new();

        for (address, class_hash) in inner_state_diff.commitment.address_to_class_hash.iter() {
            let storage_key = storage_key_build(SN_CONTRACT_CLASS_HASH_PREFIX.clone(), &address.encode());
            let storage_value = class_hash.encode();
            changes.push((storage_key, Some(storage_value)));
        }

        for (address, nonce) in inner_state_diff.commitment.address_to_nonce.iter() {
            let storage_key = storage_key_build(SN_NONCE_PREFIX.clone(), &address.encode());
            let storage_value = nonce.encode();
            changes.push((storage_key, Some(storage_value)));
        }

        for (address, storages) in inner_state_diff.commitment.storage_updates.iter() {
            for (sk, value) in storages.iter() {
                let storage_key =
                    storage_key_build(SN_STORAGE_PREFIX.clone(), &[address.encode(), sk.encode()].concat());
                let storage_value = value.encode();
                changes.push((storage_key, Some(storage_value)));
            }
        }

        for (address, compiled_class_hash) in inner_state_diff.commitment.class_hash_to_compiled_class_hash.iter() {
            let storage_key = storage_key_build(SN_COMPILED_CLASS_HASH_PREFIX.clone(), &address.encode());
            let storage_value = compiled_class_hash.encode();
            changes.push((storage_key, Some(storage_value)));
        }

        for (class_hash, contract_class) in inner_state_diff.declared_classes {
            let storage_key = storage_key_build(SN_CONTRACT_CLASS_PREFIX.clone(), &class_hash.encode());
            let storage_value = contract_class.encode();
            changes.push((storage_key, Some(storage_value)));
        }

        InnerStorageChangeSet { changes, child_changes: _child_changes }
    }
}
