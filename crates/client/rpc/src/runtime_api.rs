use blockifier::transaction::objects::TransactionExecutionInfo;
use log::error;
pub use mc_rpc_core::utils::*;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::SimulationFlags;
use mp_transactions::{HandleL1MessageTransaction, Transaction, UserTransaction};
use pallet_starknet_runtime_api::{
    ConvertTransactionRuntimeApi, StarknetRuntimeApi, StarknetTransactionExecutionError,
};
use sc_client_api::backend::Backend;
use sc_transaction_pool::ChainApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::DispatchError;
use starknet_api::api_core::{ContractAddress, EntryPointSelector};
use starknet_api::transaction::{Calldata, Event, TransactionHash};
use starknet_core::types::FieldElement;

use crate::{Starknet, StarknetRpcApiError};

type RpcApiResult<T> = Result<T, crate::errors::StarknetRpcApiError>;

impl<A, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B>,
    C: HeaderBackend<B> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    H: HasherT + Send + Sync + 'static,
{
    pub fn do_call(
        &self,
        best_block_hash: B::Hash,
        contract_address: ContractAddress,
        entry_point_selector: EntryPointSelector,
        calldata: Calldata,
    ) -> RpcApiResult<Result<Vec<Felt252Wrapper>, sp_runtime::DispatchError>> {
        self.client.runtime_api().call(best_block_hash, contract_address, entry_point_selector, calldata).map_err(|e| {
            error!("Request parameters error: {e}");
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn do_estimate_message_fee(
        &self,
        block_hash: B::Hash,
        message: HandleL1MessageTransaction,
    ) -> RpcApiResult<(u128, u64, u64)> {
        self.client
            .runtime_api()
            .estimate_message_fee(block_hash, message)
            .map_err(|e| {
                error!("Runtime Api error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Function execution failed: {:#?}", e);
                StarknetRpcApiError::ContractError
            })
    }

    pub fn do_get_tx_execution_outcome(
        &self,
        block_hash: B::Hash,
        tx_hash: TransactionHash,
    ) -> RpcApiResult<Option<Vec<u8>>> {
        self.client.runtime_api().get_tx_execution_outcome(block_hash, tx_hash).map_err(|e| {
            error!(
                "Failed to get transaction execution outcome. Substrate block hash: {block_hash}, transaction hash: \
                 {tx_hash}, error: {e}"
            );
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn do_get_events_for_tx_by_hash(
        &self,
        block_hash: B::Hash,
        tx_hash: TransactionHash,
    ) -> RpcApiResult<Vec<Event>> {
        self.client.runtime_api().get_events_for_tx_by_hash(block_hash, tx_hash).map_err(|e| {
            error!(
                "Failed to get events for transaction hash. Substrate block hash: {block_hash}, transaction hash: \
                 {tx_hash}, error: {e}"
            );
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn convert_dispatch_error(
        &self,
        best_block_hash: B::Hash,
        error: DispatchError,
    ) -> RpcApiResult<StarknetTransactionExecutionError> {
        self.client.runtime_api().convert_error(best_block_hash, error).map_err(|e| {
            error!("Failed to convert dispatch error: {:?}", e);
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn convert_tx_to_extrinsic(
        &self,
        best_block_hash: <B as BlockT>::Hash,
        transaction: UserTransaction,
    ) -> RpcApiResult<B::Extrinsic> {
        self.client.runtime_api().convert_transaction(best_block_hash, transaction).map_err(|e| {
            error!("Failed to convert transaction: {:?}", e);
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn estimate_fee(
        &self,
        block_hash: B::Hash,
        transactions: Vec<UserTransaction>,
    ) -> RpcApiResult<Vec<(u64, u64)>> {
        self.client
            .runtime_api()
            .estimate_fee(block_hash, transactions)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })
    }
    pub fn get_best_block_hash(&self) -> B::Hash {
        self.client.info().best_hash
    }

    pub fn get_chain_id(&self, block_hash: B::Hash) -> RpcApiResult<Felt252Wrapper> {
        self.client.runtime_api().chain_id(block_hash).map_err(|e| {
            error!("Failed to fetch chain_id with block_hash: {block_hash}, error: {e}");
            StarknetRpcApiError::InternalServerError
        })
    }
    pub fn filter_extrinsics(
        &self,
        block_hash: B::Hash,
        extrinsics: Vec<B::Extrinsic>,
    ) -> RpcApiResult<Vec<Transaction>> {
        self.client.runtime_api().extrinsic_filter(block_hash, extrinsics).map_err(|e| {
            error!("Failed to filter extrinsics. Substrate block hash: {block_hash}, error: {e}");
            StarknetRpcApiError::FailedToFetchPendingTransactions
        })
    }
    pub fn get_tx_messages_to_l1(
        &self,
        substrate_block_hash: B::Hash,
        transaction_hash: FieldElement,
    ) -> RpcApiResult<Vec<starknet_api::transaction::MessageToL1>> {
        self.client
            .runtime_api()
            .get_tx_messages_to_l1(substrate_block_hash, Felt252Wrapper(transaction_hash).into())
            .map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::InternalServerError
            })
    }

    pub fn is_transaction_fee_disabled(&self, substrate_block_hash: B::Hash) -> RpcApiResult<bool> {
        self.client.runtime_api().is_transaction_fee_disabled(substrate_block_hash).map_err(|e| {
            error!("Failed to get check fee disabled. Substrate block hash: {substrate_block_hash}, error: {e}");
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn simulate_tx(
        &self,
        block_hash: B::Hash,
        tx: Transaction,
        skip_validate: bool,
        skip_fee_charge: bool,
    ) -> RpcApiResult<TransactionExecutionInfo> {
        let simulations_flags = SimulationFlags { skip_validate, skip_fee_charge };
        match tx {
            Transaction::Declare(tx, contract_class) => {
                let tx = UserTransaction::Declare(tx, contract_class);
                self.simulate_user_tx(block_hash, tx, simulations_flags)
            }
            Transaction::DeployAccount(tx) => {
                let tx = UserTransaction::DeployAccount(tx);
                self.simulate_user_tx(block_hash, tx, simulations_flags)
            }
            Transaction::Invoke(tx) => {
                let tx = UserTransaction::Invoke(tx);
                self.simulate_user_tx(block_hash, tx, simulations_flags)
            }
            Transaction::L1Handler(tx) => self.simulate_l1_tx(block_hash, tx, simulations_flags),
        }
    }

    fn simulate_user_tx(
        &self,
        block_hash: B::Hash,
        tx: UserTransaction,
        simulations_flags: SimulationFlags,
    ) -> RpcApiResult<TransactionExecutionInfo> {
        // Simulate a single User Transaction
        // So the result should have single element in vector (index 0)
        self.client
            .runtime_api()
            .simulate_transactions(block_hash, vec![tx], simulations_flags)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?
            .swap_remove(0)
            .1
            .map_err(|e| {
                error!("Failed to simulate User Transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })
    }

    fn simulate_l1_tx(
        &self,
        block_hash: B::Hash,
        tx: HandleL1MessageTransaction,
        simulations_flags: SimulationFlags,
    ) -> RpcApiResult<TransactionExecutionInfo> {
        // Simulated a single HandleL1MessageTransaction
        self.client
            .runtime_api()
            .simulate_message(block_hash, tx, simulations_flags)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?
            .map_err(|e| {
                error!("Failed to simulate L1 Message: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })
    }
}
