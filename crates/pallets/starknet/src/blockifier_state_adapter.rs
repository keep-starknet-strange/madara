use core::marker::PhantomData;
use std::collections::{BTreeMap, HashMap, HashSet};

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use mp_transactions::execution::SetArbitraryNonce;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_crypto::FieldElement;

use crate::types::ContractStorageKey;
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
    visited_pcs: HashMap<ClassHash, HashSet<usize>>,
    _phantom: PhantomData<T>,
}

impl<T: Config> Default for BlockifierStateAdapter<T> {
    fn default() -> Self {
        Self {
            storage_update: BTreeMap::default(),
            class_hash_update: usize::default(),
            compiled_class_hash_update: usize::default(),
            visited_pcs: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Config> StateReader for BlockifierStateAdapter<T> {
    fn get_storage_at(&self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_storage_key: ContractStorageKey = (contract_address, key);
        Ok(Pallet::<T>::storage(contract_storage_key))
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        Ok(Pallet::<T>::nonce(contract_address))
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        Ok(ClassHash(Pallet::<T>::contract_class_hash_by_address(contract_address)))
    }

    fn get_compiled_contract_class(&self, class_hash: ClassHash) -> StateResult<ContractClass> {
        Pallet::<T>::contract_class_by_class_hash(class_hash.0).ok_or(StateError::UndeclaredClassHash(class_hash))
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Pallet::<T>::compiled_class_hash_by_class_hash(class_hash.0).ok_or(StateError::UndeclaredClassHash(class_hash))
    }
}

impl<T: Config> State for BlockifierStateAdapter<T> {
    fn set_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: StarkFelt,
    ) -> StateResult<()> {
        let contract_storage_key: ContractStorageKey = (contract_address, key);

        self.storage_update.insert(contract_storage_key, value);

        crate::StorageView::<T>::insert(contract_storage_key, value);

        Ok(())
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

        crate::ContractClassHashes::<T>::insert(contract_address, class_hash.0);

        Ok(())
    }

    fn set_contract_class(&mut self, class_hash: ClassHash, contract_class: ContractClass) -> StateResult<()> {
        println!("SET CONTRACT CLASS: {:?}", class_hash);
        crate::ContractClasses::<T>::insert(class_hash.0, contract_class);

        Ok(())
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.compiled_class_hash_update += 1;
        crate::CompiledClassHashes::<T>::insert(class_hash.0, compiled_class_hash);

        Ok(())
    }

    fn add_visited_pcs(&mut self, class_hash: ClassHash, pcs: &std::collections::HashSet<usize>) {
        self.visited_pcs.entry(class_hash).or_default().extend(pcs);
    }
}

pub struct CachedBlockifierStateAdapter<T: Config>(pub CachedState<BlockifierStateAdapter<T>>);

impl<T: Config> Default for CachedBlockifierStateAdapter<T> {
    fn default() -> Self {
        Self(CachedState::new(BlockifierStateAdapter::default(), GlobalContractCache::new(10)))
    }
}

impl<T> State for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn set_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: StarkFelt,
    ) -> StateResult<()> {
        self.0.set_storage_at(contract_address, key, value)
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        self.0.increment_nonce(contract_address)
        // let current_nonce = Pallet::<T>::nonce(contract_address);
        // let current_nonce: FieldElement = Felt252Wrapper::from(current_nonce.0).into();
        // let new_nonce: Nonce = Felt252Wrapper(current_nonce + FieldElement::ONE).into();
        // self.0.state_cache.set_nonce_value(contract_address, new_nonce);
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        self.0.set_class_hash_at(contract_address, class_hash)
    }

    fn set_contract_class(&mut self, class_hash: ClassHash, contract_class: ContractClass) -> StateResult<()> {
        self.0.set_contract_class(class_hash, contract_class)
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.0.set_compiled_class_hash(class_hash, compiled_class_hash)
    }

    fn add_visited_pcs(&mut self, class_hash: starknet_api::core::ClassHash, pcs: &std::collections::HashSet<usize>) {
        self.0.visited_pcs.entry(class_hash).or_default().extend(pcs);
    }
}

impl<T> StateReader for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn get_storage_at(&self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        self.0.get_storage_at(contract_address, key)
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.0.get_nonce_at(contract_address)
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.0.get_class_hash_at(contract_address)
    }

    fn get_compiled_contract_class(&self, class_hash: ClassHash) -> StateResult<ContractClass> {
        self.0.get_compiled_contract_class(class_hash)
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        self.0.get_compiled_class_hash(class_hash)
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

impl<T: Config> SetArbitraryNonce for BlockifierStateAdapter<T> {
    fn set_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) -> StateResult<()> {
        crate::Nonces::<T>::insert(contract_address, nonce);

        Ok(())
    }
}
