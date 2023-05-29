use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use frame_support::BoundedVec;
use madara_runtime::Runtime;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper};
use mp_starknet::storage::{
    PALLET_STARKNET, PALLET_SYSTEM, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_CURRENT_BLOCK,
    STARKNET_NONCE, SYSTEM_EVENTS,
};
use mp_starknet::transaction::types::{EventWrapper, MaxArraySize};
use pallet_starknet::types::NonceWrapper;
use pallet_starknet::Event as RuntimeEvent;
// Substrate
use sc_client_api::backend::{Backend, StorageProvider};
use scale_codec::{Decode, Encode};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_storage::StorageKey;

use super::{storage_key_build, storage_prefix_build, StorageOverride};

/// An override for runtimes that use Schema V1
pub struct SchemaV1Override<B: BlockT, C, BE> {
    client: Arc<C>,
    _marker: PhantomData<(B, BE)>,
}

impl<B: BlockT, C, BE> SchemaV1Override<B, C, BE> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: PhantomData }
    }
}

impl<B, C, BE> SchemaV1Override<B, C, BE>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
{
    fn query_storage<T: Decode + Debug>(&self, block_hash: B::Hash, key: &StorageKey) -> Option<T> {
        let data = self.client.storage(block_hash, key);
        log::info!("data: {:?}", data);
        if let Ok(Some(data)) = self.client.storage(block_hash, key) {
            let result = Decode::decode(&mut &data.0[..]);
            log::info!("result: {:?}", result);
            if let Ok(result) = result {
                return Some(result);
            }
        }
        None
    }
    fn encode_storage_key<T: Encode>(&self, key: &T) -> Vec<u8> {
        Encode::encode(key)
    }
}

impl<B, C, BE> StorageOverride<B> for SchemaV1Override<B, C, BE>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
{
    fn current_block(&self, block_hash: B::Hash) -> Option<StarknetBlock> {
        self.query_storage::<StarknetBlock>(
            block_hash,
            &StorageKey(storage_prefix_build(PALLET_STARKNET, STARKNET_CURRENT_BLOCK)),
        )
        .map(Into::into)
    }

    fn contract_class_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ContractClassWrapper> {
        let class_hash = self.contract_class_hash_by_address(block_hash, address)?;
        self.contract_class_by_class_hash(block_hash, class_hash)
    }

    fn contract_class_hash_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ClassHashWrapper> {
        let storage_contract_class_hash_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS_HASH);
        self.query_storage::<ClassHashWrapper>(
            block_hash,
            &StorageKey(storage_key_build(storage_contract_class_hash_prefix, &self.encode_storage_key(&address))),
        )
    }

    fn contract_class_by_class_hash(
        &self,
        block_hash: <B as BlockT>::Hash,
        contract_class_hash: ClassHashWrapper,
    ) -> Option<ContractClassWrapper> {
        let storage_contract_class_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS);
        self.query_storage::<ContractClassWrapper>(
            block_hash,
            &StorageKey(storage_key_build(
                storage_contract_class_prefix,
                &self.encode_storage_key(&contract_class_hash),
            )),
        )
    }

    fn nonce(&self, block_hash: <B as BlockT>::Hash, address: ContractAddressWrapper) -> Option<NonceWrapper> {
        let storage_nonce_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_NONCE);
        let nonce = self.query_storage::<NonceWrapper>(
            block_hash,
            &StorageKey(storage_key_build(storage_nonce_prefix, &self.encode_storage_key(&address))),
        );

        match nonce {
            Some(nonce) => Some(nonce),
            None => Some(NonceWrapper::default()),
        }
    }

    fn events(&self, block_hash: <B as BlockT>::Hash) -> Option<Vec<EventWrapper>> {
        let events_key = storage_prefix_build(PALLET_SYSTEM, SYSTEM_EVENTS);
        self.query_storage::<BoundedVec<RuntimeEvent<Runtime>, MaxArraySize>>(block_hash, &StorageKey(events_key));
        None
    }
}
