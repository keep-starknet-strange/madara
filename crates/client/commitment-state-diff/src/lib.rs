use core::panic;
use std::marker::PhantomData;
use std::sync::Arc;

use blockifier::state::cached_state::CommitmentStateDiff;
use futures::channel::mpsc;
use futures::StreamExt;
use indexmap::IndexMap;
use mp_hashers::HasherT;
use mp_storage::{SN_COMPILED_CLASS_HASH_PREFIX, SN_CONTRACT_CLASS_HASH_PREFIX, SN_NONCE_PREFIX, SN_STORAGE_PREFIX};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::client::BlockchainEvents;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header};
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::block::BlockHash;
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey as StarknetStorageKey;

pub struct CommitmentStateDiffWorker<B, C, H>(PhantomData<(B, C, H)>);

impl<B, C, H> CommitmentStateDiffWorker<B, C, H>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: BlockchainEvents<B>,
    C: HeaderBackend<B>,
    H: HasherT,
{
    pub async fn emit_commitment_state_diff(client: Arc<C>, mut tx: mpsc::Sender<(BlockHash, CommitmentStateDiff)>) {
        let mut storage_event_st = client
            // https://github.com/paritytech/polkadot-sdk/issues/1989
            // I can't find a way to make the child_keys logic work
            .storage_changes_notification_stream(None, None)
            .expect("the node storage changes notification stream should be up and running");

        while let Some(block_change_set) = storage_event_st.next().await {
            let starknet_block_hash = {
                let header = match client.header(block_change_set.block) {
                    Ok(opt_h) => opt_h,
                    Err(e) => {
                        log::error!(
                            "failed to interact with substrate header backend: {e}. Skipping state change gathering \
                             for substrate block with hash `{}`",
                            block_change_set.block
                        );
                        continue;
                    }
                };
                let header = match header {
                    Some(h) => h,
                    None => {
                        log::error!(
                            "no substrate block with hash `{}` in substrate header backend. Skipping state change \
                             gathering",
                            block_change_set.block
                        );
                        continue;
                    }
                };

                let digest = header.digest();
                let block = match mp_digest_log::find_starknet_block(digest) {
                    Ok(b) => b,
                    Err(e) => {
                        log::error!(
                            "failed to find a starknet block in the header's digest of substrate block with hash \
                             `{}`: {e}. Skipping state change gathering",
                            block_change_set.block
                        );
                        continue;
                    }
                };

                block.header().hash::<H>().into()
            };

            let mut commitment_state_diff = CommitmentStateDiff {
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
                    let compiled_class_hash = CompiledClassHash(match change {
                        Some(data) => StarkFelt(data.0.clone().try_into().unwrap()),
                        // This should not happen in the current state of starknet protocol, but there have been
                        // an erase_contract_class mechanism live on the network during the regenesis migration.
                        // Better safe than sorry
                        None => StarkFelt::default(),
                    });

                    commitment_state_diff.class_hash_to_compiled_class_hash.insert(class_hash, compiled_class_hash);
                }
            }

            futures::future::poll_fn(|cx| match tx.poll_ready(cx) {
                std::task::Poll::Ready(Ok(())) => {
                    std::task::Poll::Ready(tx.start_send((starknet_block_hash, commitment_state_diff.clone())))
                }
                std::task::Poll::Ready(Err(e)) => {
                    // The doc states this will happens if we drop the channel reciever
                    panic!("channel not ready: {e}");
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            })
            .await
            .expect("the channel being ready and this thread owning its only sender, this should never fail");
        }
    }
}

pub async fn log_commitment_state_diff(mut rx: mpsc::Receiver<(BlockHash, CommitmentStateDiff)>) {
    while let Some((block_hash, csd)) = rx.next().await {
        log::info!("recieved state diff for block {block_hash}: {csd:?}");
    }
}
