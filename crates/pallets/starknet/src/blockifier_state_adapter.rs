use alloc::collections::{BTreeMap, BTreeSet};
use core::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::{CommitmentStateDiff, ContractStorageKey};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use indexmap::IndexMap;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper, Felt252Wrapper};
use mp_starknet::state::StateChanges;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_crypto::FieldElement;

use crate::alloc::string::ToString;
use crate::types::{ContractStorageKeyWrapper, StorageKeyWrapper};
use crate::{Config, Pallet};

/// Empty struct that implements the traits needed by the blockifier/starknet in rust.
///
/// We feed this struct when executing a transaction so that we directly use the substrate storage
/// and not an extra layer that would add overhead.
/// We don't implement those traits directly on the pallet to avoid compilation problems.
pub struct BlockifierStateAdapter<T: Config> {
    storage_update: BTreeMap<ContractStorageKey, StarkFelt>,
    class_hash_update: usize,
    _phantom: PhantomData<T>,
}

impl<T> StateChanges for BlockifierStateAdapter<T>
where
    T: Config,
{
    fn count_state_changes(&self) -> (usize, usize, usize) {
        let keys = self.storage_update.keys();
        let n_contract_updated = BTreeSet::from_iter(keys.clone().map(|&(contract_address, _)| contract_address)).len();
        (n_contract_updated, keys.len(), self.class_hash_update)
    }
}

impl<T: Config> Default for BlockifierStateAdapter<T> {
    fn default() -> Self {
        Self { storage_update: BTreeMap::default(), class_hash_update: usize::default(), _phantom: PhantomData }
    }
}

impl<T: Config> StateReader for BlockifierStateAdapter<T> {
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();
        let key: StorageKeyWrapper = key.0.0.into();

        let contract_storage_key: ContractStorageKeyWrapper = (contract_address, key);
        let storage_content = StarkFelt::new(Pallet::<T>::storage(contract_storage_key).into())?;

        Ok(storage_content)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();

        let nonce = Nonce(StarkFelt::new(Pallet::<T>::nonce(contract_address).into())?);

        Ok(nonce)
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();

        let class_hash = ClassHash(StarkFelt::new(
            Pallet::<T>::contract_class_hash_by_address(contract_address).unwrap_or_default().into(),
        )?);

        Ok(class_hash)
    }

    fn get_compiled_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<ContractClass> {
        let wrapped_class_hash: ClassHashWrapper = class_hash.0.into();
        let opt_contract_class = Pallet::<T>::contract_class_by_class_hash(wrapped_class_hash);
        match opt_contract_class {
            Some(contract_class) => Ok(TryInto::<ContractClass>::try_into(contract_class)
                .map_err(|e| StateError::StateReadError(e.to_string()))?),
            None => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }

    fn get_compiled_class_hash(&mut self, _class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        // FIXME 708
        Ok(CompiledClassHash::default())
    }
}

impl<T: Config> State for BlockifierStateAdapter<T> {
    fn set_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        self.storage_update.insert((contract_address, key), value);
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();
        let key: StorageKeyWrapper = key.0.0.into();

        let contract_storage_key: ContractStorageKeyWrapper = (contract_address, key);

        crate::StorageView::<T>::insert(contract_storage_key, Felt252Wrapper::from(value));
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();
        let current_nonce = Pallet::<T>::nonce(contract_address);

        crate::Nonces::<T>::insert(contract_address, Felt252Wrapper(current_nonce.0 + FieldElement::ONE));

        Ok(())
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        self.class_hash_update += 1;
        let contract_address: ContractAddressWrapper = contract_address.0.0.into();
        let class_hash: ClassHashWrapper = class_hash.0.into();

        crate::ContractClassHashes::<T>::insert(contract_address, class_hash);

        Ok(())
    }

    fn set_contract_class(&mut self, class_hash: &ClassHash, contract_class: ContractClass) -> StateResult<()> {
        let class_hash: ClassHashWrapper = class_hash.0.into();
        let contract_class: ContractClassWrapper = ContractClassWrapper::try_from(contract_class).unwrap();

        crate::ContractClasses::<T>::insert(class_hash, contract_class);

        Ok(())
    }

    fn set_compiled_class_hash(
        &mut self,
        _class_hash: ClassHash,
        _compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        // FIXME 708
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
