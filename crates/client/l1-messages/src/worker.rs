use std::sync::Arc;

use ethers::core::types::Address;
use ethers::providers::{Http, Provider, StreamExt};
use madara_runtime::pallet_starknet;
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use parity_scale_codec::{Decode, Encode};
use sc_client_api::{Backend, HeaderBackend};
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::offchain::OffchainStorage;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;

use crate::config::L1MessagesWorkerConfig;
use crate::contract::{L1Contract, LogMessageToL2Filter};
use crate::error::L1MessagesWorkerError;

const TX_SOURCE: TransactionSource = TransactionSource::External;
const STORAGE_PREFIX: &str = "L1MessagesWorker";
const STORAGE_KEY: &str = "last_synced_block";

pub async fn run_worker<C, P, B, S>(config: L1MessagesWorkerConfig, client: Arc<C>, pool: Arc<P>, backend: Arc<S>)
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    S: Backend<B> + Send + 'static,
{
    log::info!("⟠ Starting L1 Messages Worker with config: {:?}", config);

    let mut offchain_storage = backend.offchain_storage().unwrap();

    let l1_contract = L1Contract::new(
        config.contract_address.parse::<Address>().unwrap(),
        Arc::new(Provider::<Http>::try_from(&config.http_provider).unwrap()),
    );

    let last_synced_block = get_block_number::<B, S>(&offchain_storage).unwrap();
    log::debug!("⟠ Last synchronized block: {:?}", last_synced_block);

    let events = l1_contract.events().from_block(last_synced_block);
    let mut stream = events.stream().await.unwrap().with_meta();
    while let Some(Ok((event, meta))) = stream.next().await {
        log::info!(
            "⟠ Processing L1 Message from block: {:?}, transaction_hash: {:?}, log_index: {:?}",
            meta.block_number,
            meta.transaction_hash,
            meta.log_index
        );

        match process_l1_message(&event, client.clone(), pool.clone()).await {
            Ok(tx_hash) => {
                log::info!(
                    "⟠ L1 Message from block: {:?}, transaction_hash: {:?}, log_index: {:?} submitted, transaction \
                     hash on L2: {:?}",
                    meta.block_number,
                    meta.transaction_hash,
                    meta.log_index,
                    tx_hash
                );
                set_block_number::<B, S>(&mut offchain_storage, &meta.block_number.as_u64());
            }
            Err(e) => {
                log::error!(
                    "Unexpected error while processing L1 Message from block: {:?}, transaction_hash: {:?}, \
                     log_index: {:?}, error: {:?}",
                    meta.block_number,
                    meta.transaction_hash,
                    meta.log_index,
                    e
                )
            }
        }
    }
}

async fn process_l1_message<C, P, B>(
    event: &LogMessageToL2Filter,
    client: Arc<C>,
    pool: Arc<P>,
) -> Result<P::Hash, L1MessagesWorkerError>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
{
    let fee = event.try_into_fee()?;
    let transaction = event.try_into_transaction()?;

    let best_block_hash = client.info().best_hash;

    match client.runtime_api().l1_nonce_unused(best_block_hash, transaction.nonce) {
        Ok(true) => Ok(()),
        Ok(false) => {
            log::debug!("⟠ Event already processed: {:?}", transaction);
            Err(L1MessagesWorkerError::MessageAlreadyProcessed)
        }
        Err(e) => {
            log::error!("⟠ Unexpected runtime api error: {:?}", e);
            Err(L1MessagesWorkerError::RuntimeApiError)
        }
    }?;

    let extrinsic = client
        .runtime_api()
        .convert_l1_transaction(best_block_hash, transaction, fee)
        .map_err(|e| {
            log::error!("⟠ Failed to convert transaction: {:?}", e);
            L1MessagesWorkerError::ConvertTransactionRuntimeApiError
        })?
        .map_err(|_| L1MessagesWorkerError::ConvertTransactionRuntimeApiError)?;

    pool.submit_one(&BlockId::Hash(best_block_hash), TX_SOURCE, extrinsic)
        .await
        .map_err(|_| L1MessagesWorkerError::SubmitTxError)
}

fn set_block_number<B, S>(storage: &mut S::OffchainStorage, blknum: &u64)
where
    B: BlockT,
    S: Backend<B> + Send + 'static,
{
    set_storage_value::<B, S, u64>(storage, STORAGE_PREFIX, STORAGE_KEY, blknum);
}

fn get_block_number<B, S>(storage: &S::OffchainStorage) -> Result<u64, L1MessagesWorkerError>
where
    B: BlockT,
    S: Backend<B> + Send + 'static,
{
    get_storage_value::<B, S, u64>(storage, STORAGE_PREFIX, STORAGE_KEY)
}

fn set_storage_value<B, S, T>(storage: &mut S::OffchainStorage, prefix: &str, key: &str, value: &T)
where
    B: BlockT,
    S: Backend<B> + Send + 'static,
    T: Encode,
{
    storage.set(prefix.as_bytes(), key.as_bytes(), &value.encode());
}

fn get_storage_value<B, S, T>(storage: &S::OffchainStorage, prefix: &str, key: &str) -> Result<T, L1MessagesWorkerError>
where
    B: BlockT,
    S: Backend<B> + Send + 'static,
    T: Decode + Default,
{
    match storage.get(prefix.as_bytes(), key.as_bytes()) {
        Some(bytes) => match T::decode(&mut &bytes[..]) {
            Ok(blknum) => Ok(blknum),
            Err(_) => Err(L1MessagesWorkerError::OffchainStorageError),
        },
        None => Ok(T::default()),
    }
}
