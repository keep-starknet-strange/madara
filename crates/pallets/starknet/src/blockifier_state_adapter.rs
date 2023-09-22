use alloc::collections::{BTreeMap, BTreeSet};
use core::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{CommitmentStateDiff, ContractStorageKey};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use mp_state::{FeeConfig, StateChanges};
use sp_core::Get;
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
    fn count_state_changes(&self) -> (usize, usize, usize, usize) {
        let keys = self.storage_update.keys();
        let n_contract_updated = BTreeSet::from_iter(keys.clone().map(|&(contract_address, _)| contract_address)).len();
        (n_contract_updated, keys.len(), self.class_hash_update, self.compiled_class_hash_update)
    }
}

impl<T> FeeConfig for BlockifierStateAdapter<T>
where
    T: Config,
{
    fn is_transaction_fee_disabled(&self) -> bool {
        T::DisableTransactionFee::get()
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
        let current_nonce: FieldElement = current_nonce.0.into();
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
