use std::marker::PhantomData;
use std::sync::Arc;

use futures::StreamExt;
use indexmap::IndexMap;
use mp_storage::{SN_COMPILED_CLASS_HASH_PREFIX, SN_CONTRACT_CLASS_HASH_PREFIX, SN_NONCE_PREFIX, SN_STORAGE_PREFIX};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::client::BlockchainEvents;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey as StarknetStorageKey;

pub struct SnosWorker<B, C>(PhantomData<(B, C)>);

impl<B, C> SnosWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
    C: HeaderBackend<B>,
{
    pub async fn run_snos(client: Arc<C>) {
        let mut storage_event_st = client
            // https://github.com/paritytech/polkadot-sdk/issues/1989
            // I can't find a way to make the child_keys logic work
            .storage_changes_notification_stream(None, None)
            .expect("the node storage changes notification stream should be up and running");

        while let Some(block_change_set) = storage_event_st.next().await {
            let block_context = match client.runtime_api().get_block_context(block_change_set.block) {
                Ok(bc) => bc,
                Err(e) => {
                    log::error!(
                        "failed to retrieve the block context from the starknet pallet runtime. Abort SNOS execution \
                         for this block. Error: {e}",
                    );
                    continue;
                }
            };

            let mut commitment_state_diff = blockifier::state::cached_state::CommitmentStateDiff {
                address_to_class_hash: Default::default(),
                address_to_nonce: Default::default(),
                storage_updates: Default::default(),
                class_hash_to_compiled_class_hash: Default::default(),
            };

            for (_prefix, full_storage_key, change) in block_change_set.changes.iter() {
                // The storages we are interested in here all have longe keys
                if full_storage_key.0.len() < 32 {
                    continue;
                }
                let prefix = &full_storage_key.0[..32];

                // All the try_into are safe to unwrap because we know that is what the storage contains
                // and therefore what length it is
                if prefix == *SN_NONCE_PREFIX {
                    let contract_address =
                        ContractAddress(PatriciaKey(StarkFelt(full_storage_key.0[32..].try_into().unwrap())));
                    // `change` is safe to unwrap as `Nonces` storage is `ValueQuery`
                    let nonce = Nonce(StarkFelt(change.unwrap().0.clone().try_into().unwrap()));
                    commitment_state_diff.address_to_nonce.insert(contract_address, nonce);
                } else if prefix == *SN_STORAGE_PREFIX {
                    let contract_address =
                        ContractAddress(PatriciaKey(StarkFelt(full_storage_key.0[32..64].try_into().unwrap())));
                    let storage_key =
                        StarknetStorageKey(PatriciaKey(StarkFelt(full_storage_key.0[64..].try_into().unwrap())));
                    // `change` is safe to unwrap as `StorageView` storage is `ValueQuery`
                    let value = StarkFelt(change.unwrap().0.clone().try_into().unwrap());

                    match commitment_state_diff.storage_updates.get_mut(&contract_address) {
                        Some(contract_storage) => {
                            contract_storage.insert(storage_key, value);
                        }
                        None => {
                            let mut contract_storage: IndexMap<_, _, _> = Default::default();
                            contract_storage.insert(storage_key, value);

                            commitment_state_diff.storage_updates.insert(contract_address, contract_storage);
                        }
                    }
                } else if prefix == *SN_CONTRACT_CLASS_HASH_PREFIX {
                    let contract_address =
                        ContractAddress(PatriciaKey(StarkFelt(full_storage_key.0[32..].try_into().unwrap())));
                    // `change` is safe to unwrap as `ContractClassHashes` storage is `ValueQuery`
                    let class_hash = ClassHash(StarkFelt(change.unwrap().0.clone().try_into().unwrap()));

                    commitment_state_diff.address_to_class_hash.insert(contract_address, class_hash);
                } else if prefix == *SN_COMPILED_CLASS_HASH_PREFIX {
                    let class_hash = ClassHash(StarkFelt(full_storage_key.0[32..].try_into().unwrap()));
                    // `change` is safe to unwrap, despite `CompiledClassHashes` being an `OptionQuery`,
                    // because the starknet protocol guarantee that its storage values
                    // are never erased (set to `None` again)
                    let compiled_class_hash =
                        CompiledClassHash(StarkFelt(change.unwrap().0.clone().try_into().unwrap()));

                    commitment_state_diff.class_hash_to_compiled_class_hash.insert(class_hash, compiled_class_hash);
                }
            }

            println!("commitment: {:?}, block context: {:?}", commitment_state_diff, block_context);

            // TODO: call snos
        }
    }
}
