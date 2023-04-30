use core::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper};
use sp_core::{H256, U256};
use sp_std::sync::Arc;
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::{StateDiff, StorageKey};

use crate::alloc::string::ToString;
use crate::types::{ContractStorageKeyWrapper, StorageKeyWrapper};
use crate::{Config, Pallet};

/// Empty struct that implements the traits needed by the blockifier/starknet in rust.
///
/// We feed this struct when executing a transaction so that we directly use the substrate storage
/// and not an extra layer that would add overhead.
/// We don't implement those traits directly on the pallet to avoid compilation problems.
pub struct BlockifierStateAdapter<T: Config>(PhantomData<T>);
impl<T: Config> Default for BlockifierStateAdapter<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Config> StateReader for BlockifierStateAdapter<T> {
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;
        let key: StorageKeyWrapper = H256::from(key.0.0.0);

        let contract_storage_key: ContractStorageKeyWrapper = (contract_address, key);
        let storage_content = StarkFelt::new(Pallet::<T>::storage(contract_storage_key).into())?;

        Ok(storage_content)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;

        let nonce = Nonce(StarkFelt::new(Pallet::<T>::nonce(contract_address).into())?);

        Ok(nonce)
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;

        let class_hash = ClassHash(StarkFelt::new(
            Pallet::<T>::contract_class_hash_by_address(contract_address).unwrap_or_default(),
        )?);

        Ok(class_hash)
    }

    fn get_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<Arc<ContractClass>> {
        let wrapped_class_hash: ClassHashWrapper = class_hash.0.0;

        let opt_contract_class = Pallet::<T>::contract_class_by_class_hash(wrapped_class_hash);
        match opt_contract_class {
            Some(contract_class) => Ok(Arc::new(
                TryInto::<ContractClass>::try_into(contract_class)
                    .map_err(|e| StateError::StateReadError(e.to_string()))?,
            )),
            None => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }
}

impl<T: Config> State for BlockifierStateAdapter<T> {
    fn set_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey, value: StarkFelt) {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;
        let key: StorageKeyWrapper = H256::from(key.0.0.0);

        let contract_storage_key: ContractStorageKeyWrapper = (contract_address, key);

        crate::StorageView::<T>::insert(contract_storage_key, U256::from(value.0));
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        let current_nonce = Pallet::<T>::nonce(contract_address.0.0.0);

        crate::Nonces::<T>::insert(contract_address.0.0.0, current_nonce + 1);

        Ok(())
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;
        let class_hash: ClassHashWrapper = class_hash.0.0;

        crate::ContractClassHashes::<T>::insert(contract_address, class_hash);

        Ok(())
    }

    fn set_contract_class(&mut self, class_hash: &ClassHash, contract_class: ContractClass) -> StateResult<()> {
        let class_hash: ClassHashWrapper = class_hash.0.0;
        let contract_class: ContractClassWrapper = ContractClassWrapper::try_from(contract_class).unwrap();

        crate::ContractClasses::<T>::insert(class_hash, contract_class);

        Ok(())
    }

    /// As the state is updated during the execution, return an empty [StateDiff]
    ///
    /// There is no reason to use it in the current implementation of the trait
    fn to_state_diff(&self) -> StateDiff {
        StateDiff::default()
    }
}
