use std::marker::PhantomData;
use std::sync::Arc;

use blockifier::execution::contract_class::ContractClass;
use mp_storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_NONCE, STARKNET_STORAGE,
    STARKNET_TX_EVENTS,
};
use parity_scale_codec::{Decode, Encode};
// Substrate
use sc_client_api::backend::{Backend, StorageProvider};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_storage::StorageKey;
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey as StarknetStorageKey;
use starknet_api::transaction::{Event as StarknetEvent, TransactionHash};

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
    fn query_storage<T: Decode>(&self, block_hash: B::Hash, key: &StorageKey) -> Option<T> {
        if let Ok(Some(data)) = self.client.storage(block_hash, key) {
            if let Ok(result) = Decode::decode(&mut &data.0[..]) {
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
    fn get_storage_by_storage_key(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddress,
        key: StarknetStorageKey,
    ) -> Option<StarkFelt> {
        let storage_storage_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_STORAGE);
        let key = (address, key);

        // check if contract exists
        match self.contract_class_hash_by_address(block_hash, address) {
            Some(_) => (),
            None => return None,
        }

        let storage = self.query_storage::<StarkFelt>(
            block_hash,
            &StorageKey(storage_key_build(storage_storage_prefix, &self.encode_storage_key(&key))),
        );

        match storage {
            Some(storage) => Some(storage),
            None => Some(Default::default()),
        }
    }

    fn contract_class_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddress,
    ) -> Option<ContractClass> {
        let class_hash = self.contract_class_hash_by_address(block_hash, address)?;
        self.contract_class_by_class_hash(block_hash, class_hash)
    }

    fn contract_class_hash_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddress,
    ) -> Option<ClassHash> {
        let storage_contract_class_hash_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS_HASH);
        self.query_storage::<ClassHash>(
            block_hash,
            &StorageKey(storage_key_build(storage_contract_class_hash_prefix, &self.encode_storage_key(&address))),
        )
    }

    fn contract_class_by_class_hash(
        &self,
        block_hash: <B as BlockT>::Hash,
        contract_class_hash: ClassHash,
    ) -> Option<ContractClass> {
        let storage_contract_class_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS);
        self.query_storage::<ContractClass>(
            block_hash,
            &StorageKey(storage_key_build(
                storage_contract_class_prefix,
                &self.encode_storage_key(&contract_class_hash),
            )),
        )
    }

    fn nonce(&self, block_hash: <B as BlockT>::Hash, address: ContractAddress) -> Option<Nonce> {
        self.contract_class_hash_by_address(block_hash, address)?;

        let storage_nonce_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_NONCE);
        let nonce = self.query_storage::<Nonce>(
            block_hash,
            &StorageKey(storage_key_build(storage_nonce_prefix, &self.encode_storage_key(&address))),
        );

        match nonce {
            Some(nonce) => Some(nonce),
            None => Some(Nonce::default()),
        }
    }

    fn get_events_for_tx_by_hash(
        &self,
        block_hash: <B as BlockT>::Hash,
        tx_hash: TransactionHash,
    ) -> Option<Vec<StarknetEvent>> {
        let storage_tx_events_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_TX_EVENTS);
        self.query_storage::<Vec<StarknetEvent>>(
            block_hash,
            &StorageKey(storage_key_build(storage_tx_events_prefix, &self.encode_storage_key(&tx_hash))),
        )
    }
}
