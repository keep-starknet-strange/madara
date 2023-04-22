use core::marker::PhantomData;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{StateReader, StateResult};
use mp_starknet::execution::{ClassHashWrapper, ContractAddressWrapper};
use sp_core::H256;
use sp_std::sync::Arc;
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;

use crate::alloc::string::ToString;
use crate::types::{ContractStorageKeyWrapper, StorageKeyWrapper};
use crate::{Config, Pallet};

/// A zero-sized struct to implement StateReader on while being generic over `T`
pub struct BLockifierStateReader<T> {
    marker: PhantomData<T>,
}

impl<T> BLockifierStateReader<T> {
    pub fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<T> Default for BLockifierStateReader<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config> StateReader for BLockifierStateReader<T> {
    fn get_storage_at(&mut self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;
        let key: StorageKeyWrapper = H256::from(key.0.0.0);

        let contract_storage_key: ContractStorageKeyWrapper = (contract_address, key);
        let value = StarkFelt::new(Pallet::<T>::storage(contract_storage_key).into())?;

        Ok(value)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;

        let nonce = Nonce(StarkFelt::new(Pallet::<T>::nonce(contract_address).into())?);

        Ok(nonce)
    }

    fn get_contract_class(&mut self, class_hash: &ClassHash) -> StateResult<Arc<ContractClass>> {
        let wrapped_class_hash: ClassHashWrapper = class_hash.0.0;

        let opt_contract_class = Pallet::<T>::contract_class_by_class_hash(wrapped_class_hash);
        match opt_contract_class {
            Some(contract_class) => Ok(Arc::new(
                contract_class.to_starknet_contract_class().map_err(|e| StateError::StateReadError(e.to_string()))?,
            )),
            None => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let contract_address: ContractAddressWrapper = contract_address.0.0.0;

        let class_hash = ClassHash(StarkFelt::new(
            Pallet::<T>::contract_class_hash_by_address(contract_address).unwrap_or_default(),
        )?);

        Ok(class_hash)
    }
}
