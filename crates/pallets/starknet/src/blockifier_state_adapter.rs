use alloc::collections::{BTreeMap, BTreeSet};
use core::marker::PhantomData;
use std::collections::HashMap;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{CommitmentStateDiff, ContractStorageKey, StateChangesCount, StorageView};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use mp_state::StateChanges;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_crypto::FieldElement;

use crate::{Config, Pallet};

/// Empty struct that implements the traits needed by the blockifier/starknet in rust.
///
/// We feed this struct when executing a transaction so that we directly use the substrate storage
/// and not an extra layer that would add overhead.
/// We don't implement those traits directly on the pallet to avoid compilation problems.
pub struct BlockifierStateAdapter<T: Config> {
    storage_update: BTreeMap<ContractStorageKey, StarkFelt>,
    class_hash_update: usize,
    compiled_class_hash_update: usize,
    state_cache: StateCache,
    _phantom: PhantomData<T>,
}

impl<T> StateChanges for BlockifierStateAdapter<T>
where
    T: Config,
{
    fn count_state_changes(&self) -> StateChangesCount {
        let keys = self.storage_update.keys();
        let n_contract_updated = BTreeSet::from_iter(keys.clone().map(|&(contract_address, _)| contract_address)).len();
        StateChangesCount {
            n_modified_contracts: n_contract_updated,
            n_storage_updates: keys.len(),
            n_class_hash_updates: self.class_hash_update,
            n_compiled_class_hash_updates: self.compiled_class_hash_update,
        }
    }
}

impl<T: Config> Default for BlockifierStateAdapter<T> {
    fn default() -> Self {
        Self {
            storage_update: BTreeMap::default(),
            class_hash_update: usize::default(),
            compiled_class_hash_update: usize::default(),
            state_cache: StateCache::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Config> StateReader for BlockifierStateAdapter<T> {
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_storage_key: ContractStorageKey = (contract_address, key);
        Ok(Pallet::<T>::storage(contract_storage_key))
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        Ok(Pallet::<T>::nonce(contract_address))
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        Ok(Pallet::<T>::contract_class_hash_by_address(contract_address))
    }

    fn get_compiled_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<ContractClass> {
        Pallet::<T>::contract_class_by_class_hash(class_hash).ok_or(StateError::UndeclaredClassHash(*class_hash))
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Pallet::<T>::compiled_class_hash_by_class_hash(class_hash).ok_or(StateError::UndeclaredClassHash(class_hash))
    }
}

impl<T: Config> State for BlockifierStateAdapter<T> {
    fn set_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        let contract_storage_key: ContractStorageKey = (contract_address, key);

        self.storage_update.insert(contract_storage_key, value);

        crate::StorageView::<T>::insert(contract_storage_key, value);
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        let current_nonce = Pallet::<T>::nonce(contract_address);
        let current_nonce: FieldElement = Felt252Wrapper::from(current_nonce.0).into();
        let new_nonce: Nonce = Felt252Wrapper(current_nonce + FieldElement::ONE).into();

        crate::Nonces::<T>::insert(contract_address, new_nonce);

        Ok(())
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        self.class_hash_update += 1;

        crate::ContractClassHashes::<T>::insert(contract_address, class_hash);

        Ok(())
    }

    fn set_contract_class(&mut self, class_hash: &ClassHash, contract_class: ContractClass) -> StateResult<()> {
        crate::ContractClasses::<T>::insert(class_hash, contract_class);

        Ok(())
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.compiled_class_hash_update += 1;
        crate::CompiledClassHashes::<T>::insert(class_hash, compiled_class_hash);

        Ok(())
    }

    /// As the state is updated during the execution, return an empty [StateDiff]
    ///
    /// There is no reason to use it in the current implementation of the trait
    fn to_state_diff(&self) -> CommitmentStateDiff {
        CommitmentStateDiff {
            address_to_class_hash: IndexMap::with_capacity_and_hasher(0, Default::default()),
            address_to_nonce: IndexMap::with_capacity_and_hasher(0, Default::default()),
            storage_updates: IndexMap::with_capacity_and_hasher(0, Default::default()),
            class_hash_to_compiled_class_hash: IndexMap::with_capacity_and_hasher(0, Default::default()),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct StateCache {
    // Reader's cached information; initial values, read before any write operation (per cell).
    nonce_initial_values: IndexMap<ContractAddress, Nonce>,
    class_hash_initial_values: IndexMap<ContractAddress, ClassHash>,
    storage_initial_values: IndexMap<ContractStorageKey, StarkFelt>,
    compiled_class_hash_initial_values: IndexMap<ClassHash, CompiledClassHash>,

    // Writer's cached information.
    nonce_writes: IndexMap<ContractAddress, Nonce>,
    class_hash_writes: IndexMap<ContractAddress, ClassHash>,
    storage_writes: IndexMap<ContractStorageKey, StarkFelt>,
    compiled_class_hash_writes: IndexMap<ClassHash, CompiledClassHash>,
}

impl StateCache {
    fn get_storage_at(&self, contract_address: ContractAddress, key: StorageKey) -> Option<&StarkFelt> {
        let contract_storage_key = (contract_address, key);
        self.storage_writes
            .get(&contract_storage_key)
            .or_else(|| self.storage_initial_values.get(&contract_storage_key))
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> Option<&Nonce> {
        self.nonce_writes.get(&contract_address).or_else(|| self.nonce_initial_values.get(&contract_address))
    }

    pub fn set_storage_initial_value(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        let contract_storage_key = (contract_address, key);
        self.storage_initial_values.insert(contract_storage_key, value);
    }

    fn set_storage_value(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        let contract_storage_key = (contract_address, key);
        self.storage_writes.insert(contract_storage_key, value);
    }

    fn set_nonce_initial_value(&mut self, contract_address: ContractAddress, nonce: Nonce) {
        self.nonce_initial_values.insert(contract_address, nonce);
    }

    fn set_nonce_value(&mut self, contract_address: ContractAddress, nonce: Nonce) {
        self.nonce_writes.insert(contract_address, nonce);
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> Option<&ClassHash> {
        self.class_hash_writes.get(&contract_address).or_else(|| self.class_hash_initial_values.get(&contract_address))
    }

    fn set_class_hash_initial_value(&mut self, contract_address: ContractAddress, class_hash: ClassHash) {
        self.class_hash_initial_values.insert(contract_address, class_hash);
    }

    fn set_class_hash_write(&mut self, contract_address: ContractAddress, class_hash: ClassHash) {
        self.class_hash_writes.insert(contract_address, class_hash);
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> Option<&CompiledClassHash> {
        self.compiled_class_hash_writes
            .get(&class_hash)
            .or_else(|| self.compiled_class_hash_initial_values.get(&class_hash))
    }

    fn set_compiled_class_hash_initial_value(&mut self, class_hash: ClassHash, compiled_class_hash: CompiledClassHash) {
        self.compiled_class_hash_initial_values.insert(class_hash, compiled_class_hash);
    }

    fn set_compiled_class_hash_write(&mut self, class_hash: ClassHash, compiled_class_hash: CompiledClassHash) {
        self.compiled_class_hash_writes.insert(class_hash, compiled_class_hash);
    }

    fn get_storage_updates(&self) -> HashMap<ContractStorageKey, StarkFelt> {
        HashMap::from_iter(subtract_mappings(&self.storage_writes, &self.storage_initial_values))
    }

    fn get_class_hash_updates(&self) -> IndexMap<ContractAddress, ClassHash> {
        subtract_mappings(&self.class_hash_writes, &self.class_hash_initial_values)
    }

    fn get_nonce_updates(&self) -> IndexMap<ContractAddress, Nonce> {
        subtract_mappings(&self.nonce_writes, &self.nonce_initial_values)
    }

    fn get_compiled_class_hash_updates(&self) -> IndexMap<ClassHash, CompiledClassHash> {
        subtract_mappings(&self.compiled_class_hash_writes, &self.compiled_class_hash_initial_values)
    }
}

pub struct CachedBlockifierStateAdapter<T: Config>(pub BlockifierStateAdapter<T>);

impl<T> StateChanges for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn count_state_changes(&self) -> StateChangesCount {
        self.0.count_state_changes()
    }
}

impl<T> State for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn set_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        self.0.state_cache.set_storage_value(contract_address, key, value);
        self.0.set_storage_at(contract_address, key, value);
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        let current_nonce = Pallet::<T>::nonce(contract_address);
        let current_nonce: FieldElement = Felt252Wrapper::from(current_nonce.0).into();
        let new_nonce: Nonce = Felt252Wrapper(current_nonce + FieldElement::ONE).into();
        self.0.state_cache.set_nonce_value(contract_address, new_nonce);
        self.0.increment_nonce(contract_address)
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        self.0.state_cache.set_class_hash_write(contract_address, class_hash);
        self.0.set_class_hash_at(contract_address, class_hash)
    }

    fn set_contract_class(&mut self, class_hash: &ClassHash, contract_class: ContractClass) -> StateResult<()> {
        self.0.set_contract_class(class_hash, contract_class)
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.0.state_cache.set_compiled_class_hash_write(class_hash, compiled_class_hash);
        self.0.set_compiled_class_hash(class_hash, compiled_class_hash)
    }

    fn to_state_diff(&self) -> CommitmentStateDiff {
        type StorageDiff = IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>>;

        let state_cache = &self.0.state_cache;
        let class_hash_updates = state_cache.get_class_hash_updates();
        let storage_diffs = state_cache.get_storage_updates();
        let nonces = state_cache.get_nonce_updates();
        let declared_classes = state_cache.get_compiled_class_hash_updates();

        CommitmentStateDiff {
            address_to_class_hash: class_hash_updates,
            storage_updates: StorageDiff::from(StorageView(storage_diffs)),
            class_hash_to_compiled_class_hash: declared_classes,
            address_to_nonce: nonces,
        }
    }
}

impl<T> StateReader for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let storage_value = self.0.get_storage_at(contract_address, key)?;
        if self.0.state_cache.get_storage_at(contract_address, key).is_none() {
            self.0.state_cache.set_storage_initial_value(contract_address, key, storage_value);
        }
        Ok(storage_value)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let nonce = self.0.get_nonce_at(contract_address)?;
        if self.0.state_cache.get_nonce_at(contract_address).is_none() {
            self.0.state_cache.set_nonce_initial_value(contract_address, nonce);
        }
        Ok(nonce)
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let class_hash = self.0.get_class_hash_at(contract_address)?;
        if self.0.state_cache.get_class_hash_at(contract_address).is_none() {
            self.0.state_cache.set_class_hash_initial_value(contract_address, class_hash);
        }
        Ok(class_hash)
    }

    fn get_compiled_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<ContractClass> {
        self.0.get_compiled_contract_class(class_hash)
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        let compiled_class_hash = self.0.get_compiled_class_hash(class_hash)?;
        if self.0.state_cache.get_compiled_class_hash(class_hash).is_none() {
            self.0.state_cache.set_compiled_class_hash_initial_value(class_hash, compiled_class_hash);
        }
        Ok(compiled_class_hash)
    }
}

/// Returns a `IndexMap` containing key-value pairs from `a` that are not included in `b` (if
/// a key appears in `b` with a different value, it will be part of the output).
/// Usage: Get updated items from a mapping.
pub fn subtract_mappings<K, V>(lhs: &IndexMap<K, V>, rhs: &IndexMap<K, V>) -> IndexMap<K, V>
where
    K: Clone + Eq + core::hash::Hash,
    V: Clone + PartialEq,
{
    lhs.iter().filter(|(k, v)| rhs.get(*k) != Some(v)).map(|(k, v)| (k.clone(), v.clone())).collect()
}
