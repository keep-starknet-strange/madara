use std::marker::PhantomData;
use std::sync::Arc;

use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper};
use mp_starknet::storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_CURRENT_BLOCK,
};
// Substrate
use sc_client_api::backend::{Backend, StorageProvider};
use scale_codec::Decode;
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
    fn query_storage<T: Decode>(&self, block_hash: B::Hash, key: &StorageKey) -> Option<T> {
        if let Ok(Some(data)) = self.client.storage(block_hash, key) {
            if let Ok(result) = Decode::decode(&mut &data.0[..]) {
                return Some(result);
            }
        }
        None
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

    fn contract_class(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ContractClassWrapper> {
        let storage_contract_class_hash_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS_HASH);
        let contract_class_hash: [u8; 32] = self.query_storage::<ClassHashWrapper>(
            block_hash,
            &StorageKey(storage_key_build(storage_contract_class_hash_prefix, &address[..])),
        )?;

        let storage_contract_class_prefix = storage_prefix_build(PALLET_STARKNET, STARKNET_CONTRACT_CLASS);
        self.query_storage::<ContractClassWrapper>(
            block_hash,
            &StorageKey(storage_key_build(storage_contract_class_prefix, &contract_class_hash[..])),
        )
    }
}
