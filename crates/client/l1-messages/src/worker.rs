use std::sync::Arc;

use ethers::providers::{Http, Provider, StreamExt};
use madara_runtime::pallet_starknet;
use mp_transactions::HandleL1MessageTransaction;
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use parity_scale_codec::{Decode, Encode};
use sc_client_api::{Backend, HeaderBackend};
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::offchain::OffchainStorage;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use starknet_api::transaction::Fee;

use crate::config::L1MessagesWorkerConfig;
use crate::contract::{L1Contract, LogMessageToL2Filter};
use crate::error::L1MessagesWorkerError;

const TX_SOURCE: TransactionSource = TransactionSource::External;
const STORAGE_PREFIX: &str = "L1MessagesWorker";
const STORAGE_KEY: &str = "last_synced_block";

pub fn connect_to_l1_contract(
    config: &L1MessagesWorkerConfig,
) -> Result<L1Contract<Provider<Http>>, L1MessagesWorkerError> {
    let provider = Provider::<Http>::try_from(config.get_provider()).map_err(|e| {
        log::error!("⟠ Failed to connect to L1 Node: {:?}", e);
        L1MessagesWorkerError::ConfigError
    })?;

    let l1_contract = L1Contract::new(*config.get_contract_address(), Arc::new(provider));

    Ok(l1_contract)
}

pub async fn run_worker<C, P, B, S>(config: L1MessagesWorkerConfig, client: Arc<C>, pool: Arc<P>, backend: Arc<S>)
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    S: Backend<B> + Send + 'static,
{
    log::info!("⟠ Starting L1 Messages Worker with settings: {:?}", config);

    let mut offchain_storage = match backend.offchain_storage() {
        Some(offchain_storage) => offchain_storage,
        None => {
            log::error!("⟠ Offchain storage unavailable");
            return;
        }
    };

    let l1_contract = match connect_to_l1_contract(&config) {
        Ok(l1_contract) => l1_contract,
        Err(e) => {
            log::error!("⟠ Unexpected error while connecting to L1: {:?}", e);
            return;
        }
    };

    let events = l1_contract.events().from_block(get_block_number::<B, S>(&offchain_storage).unwrap_or_default());
    let mut stream = match events.stream().await {
        Ok(stream) => stream.with_meta(),
        Err(e) => {
            log::error!("⟠ Unexpected error with L1 event stream: {:?}", e);
            return;
        }
    };

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
                    "⟠ Unexpected error while processing L1 Message from block: {:?}, transaction_hash: {:?}, \
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
    let fee: Fee = event.try_get_fee()?;
    let transaction: HandleL1MessageTransaction = event.try_into()?;

    let best_block_hash = client.info().best_hash;

    match client.runtime_api().l1_nonce_unused(best_block_hash, transaction.nonce) {
        Ok(true) => Ok(()),
        Ok(false) => {
            log::debug!("⟠ Event already processed: {:?}", transaction);
            Err(L1MessagesWorkerError::L1MessageAlreadyProcessed)
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
            log::error!("⟠ Failed to convert transaction via runtime api: {:?}", e);
            L1MessagesWorkerError::ConvertTransactionRuntimeApiError
        })?
        .map_err(|e| {
            log::error!("⟠ Failed to convert transaction via runtime api: {:?}", e);
            L1MessagesWorkerError::ConvertTransactionRuntimeApiError
        })?;

    pool.submit_one(best_block_hash, TX_SOURCE, extrinsic).await.map_err(|e| {
        log::error!("⟠ Failed to submit transaction with L1 Message: {:?}", e);
        L1MessagesWorkerError::SubmitTxError
    })
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
