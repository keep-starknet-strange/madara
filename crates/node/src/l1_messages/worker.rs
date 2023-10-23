use mp_felt::Felt252Wrapper;

use crate::l1_messages::error::L1MessagesWorkerError;

pub(crate) const LOG_TARGET: &str = "node::service::L1MessagesWorker ⟠";

use std::sync::Arc;

use ethers::contract::abigen;
use ethers::core::types::Address;
use ethers::providers::{Http, Provider, StreamExt};
use ethers::types::U256;
use madara_runtime::pallet_starknet;
use mp_transactions::HandleL1MessageTransaction;
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use parity_scale_codec::{Decode, Encode};
use sc_client_api::{Backend, HeaderBackend};
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::offchain::OffchainStorage;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;
use starknet_api::api_core::Nonce;
use starknet_api::transaction::Fee;

const TX_SOURCE: TransactionSource = TransactionSource::External;
const STORAGE_PREFIX: &str = "L1MessagesWorker";
const STORAGE_KEY: &str = "last_synced_block";

const L1_MESSAGES_CONTRACT_ADDRESS: &str = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
abigen!(
    L1Contract,
    r"[
	event LogMessageToL2(address indexed fromAddress, uint256 indexed toAddress, uint256 indexed selector, uint256[] payload, uint256 nonce, uint256 fee)
]"
);

impl LogMessageToL2Filter {
    fn try_into_fee(&self) -> Result<Fee, L1MessagesWorkerError> {
        // Check against panic
        // https://docs.rs/ethers/latest/ethers/types/struct.U256.html#method.as_u128
        if self.fee > U256::from_big_endian(&(u128::MAX.to_be_bytes())) {
            Err(L1MessagesWorkerError::ToTransactionError)
        } else {
            Ok(Fee(self.fee.as_u128()))
        }
    }
    fn try_into_transaction(&self) -> Result<HandleL1MessageTransaction, L1MessagesWorkerError> {
        // L2 contract to call.
        let contract_address = Felt252Wrapper::try_from(sp_core::U256(self.to_address.0))?;

        // Function of the contract to call.
        let entry_point_selector = Felt252Wrapper::try_from(sp_core::U256(self.selector.0))?;

        // L1 message nonce.
        let nonce: u64 = Felt252Wrapper::try_from(sp_core::U256(self.nonce.0))?.try_into()?;

        // Add the from address here so it's directly in the calldata.
        let mut calldata: Vec<Felt252Wrapper> = Vec::from([Felt252Wrapper::try_from(self.from_address.as_bytes())?]);

        for x in &self.payload {
            calldata.push(Felt252Wrapper::try_from(sp_core::U256(x.0))?);
        }

        let tx = HandleL1MessageTransaction { nonce, contract_address, entry_point_selector, calldata };
        Ok(tx)
    }
}
pub async fn run_worker<C, P, B, S>(client: Arc<C>, pool: Arc<P>, backend: Arc<S>)
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    S: Backend<B> + Send + 'static,
{
    let mut offchain_storage = backend.offchain_storage().unwrap();

    log::info!(target: LOG_TARGET,"Starting L1 Messages Worker!");
    let eth_client = Arc::new(Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap());

    let l1_contract_address = L1_MESSAGES_CONTRACT_ADDRESS.parse::<Address>().unwrap();
    let l1_contract = L1Contract::new(l1_contract_address, Arc::clone(&eth_client));

    let last_synced_block = get_block_number::<B, S>(&offchain_storage).unwrap();
    log::info!("Last synchronized block: {:?}", last_synced_block);

    let events = l1_contract.events().from_block(last_synced_block);
    let mut stream = events.stream().await.unwrap().with_meta();
    while let Some(Ok((event, meta))) = stream.next().await {
        log::info!(target: LOG_TARGET,
            "Processing event, block_number: {:?}, block_hash: {:?}, transaction_hash: {:?}, transaction_index: {:?}, \
             log_index: {:?}",
            meta.block_number,
            meta.block_hash,
            meta.transaction_hash,
            meta.transaction_index,
            meta.log_index
        );

        let fee = event.try_into_fee().unwrap();
        let transaction = event.try_into_transaction().unwrap();

        log::info!(target: LOG_TARGET, "Transaction: {:?}", transaction);

        let best_block_hash = client.info().best_hash;

        if client
            .runtime_api()
            .ensure_l1_nonce_unused(
                best_block_hash,
                &Nonce(starknet_api::hash::StarkFelt::try_from(transaction.nonce).unwrap()),
            )
            .unwrap()
        {
            log::info!("Event already processed: {:?}", transaction);
            continue;
        }

        let extrinsic = client
            .runtime_api()
            .convert_l1_transaction(best_block_hash, transaction, fee)
            .map_err(|e| {
                log::error!("Failed to convert transaction: {:?}", e);
                L1MessagesWorkerError::ToTransactionError
            })
            .unwrap()
            .unwrap();

        let result = pool.submit_one(&BlockId::Hash(best_block_hash), TX_SOURCE, extrinsic).await;

        log::info!("Transaction submission result: {:?}", result);
        set_block_number::<B, S>(&mut offchain_storage, &meta.block_number.as_u64());
    }
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
