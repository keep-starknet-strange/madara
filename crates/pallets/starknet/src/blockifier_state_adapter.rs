use alloc::collections::{BTreeMap, BTreeSet};
use core::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{CachedState, CommitmentStateDiff, ContractStorageKey, StateChangesCount};
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

pub struct CachedBlockifierStateAdapter<T: Config>(pub CachedState<BlockifierStateAdapter<T>>);

impl<T> StateChanges for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn count_state_changes(&self) -> StateChangesCount {
        self.0.state.count_state_changes()
    }
}

impl<T> State for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn set_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        self.0.set_storage_at(contract_address, key, value);
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        self.0.increment_nonce(contract_address)
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
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
        self.0.set_compiled_class_hash(class_hash, compiled_class_hash)
    }

    fn to_state_diff(&self) -> CommitmentStateDiff {
        self.0.to_state_diff()
    }
}

impl<T> StateReader for CachedBlockifierStateAdapter<T>
where
    T: Config,
{
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        self.0.get_storage_at(contract_address, key)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.0.get_nonce_at(contract_address)
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.0.get_class_hash_at(contract_address)
    }

    fn get_compiled_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<ContractClass> {
        self.0.get_compiled_contract_class(class_hash)
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        self.0.get_compiled_class_hash(class_hash)
    }
}
