use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::L1HandlerTransaction;
use log::error;
pub use mc_rpc_core::utils::*;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::SimulationFlags;
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_block_builder::GetPendingBlockExtrinsics;
use sc_client_api::backend::Backend;
use sc_transaction_pool::ChainApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::transaction::{Calldata, Event, TransactionHash};
use starknet_core::types::FeeEstimate;

use crate::{Starknet, StarknetRpcApiError};

type RpcApiResult<T> = Result<T, crate::errors::StarknetRpcApiError>;

impl<A, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B>,
    C: HeaderBackend<B> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    H: HasherT + Send + Sync + 'static,
{
    pub fn do_call(
        &self,
        best_block_hash: B::Hash,
        contract_address: ContractAddress,
        entry_point_selector: EntryPointSelector,
        calldata: Calldata,
    ) -> RpcApiResult<Vec<Felt252Wrapper>> {
        Ok(self.client.runtime_api().call(best_block_hash, contract_address, entry_point_selector, calldata).map_err(
            |e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            },
        )??)
    }

    pub fn do_estimate_message_fee(
        &self,
        block_hash: B::Hash,
        message: L1HandlerTransaction,
    ) -> RpcApiResult<FeeEstimate> {
        Ok((&self.client.runtime_api().estimate_message_fee(block_hash, message).map_err(|e| {
            error!("Runtime Api error: {e}");
            StarknetRpcApiError::InternalServerError
        })???)
            .into())
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

    pub fn convert_tx_to_extrinsic(
        &self,
        best_block_hash: <B as BlockT>::Hash,
        transaction: AccountTransaction,
    ) -> RpcApiResult<B::Extrinsic> {
        self.client.runtime_api().convert_account_transaction(best_block_hash, transaction).map_err(|e| {
            error!("Failed to convert transaction: {:?}", e);
            StarknetRpcApiError::InternalServerError
        })
    }

    pub fn estimate_fee(
        &self,
        block_hash: B::Hash,
        transactions: Vec<AccountTransaction>,
        simulation_flags: SimulationFlags,
    ) -> RpcApiResult<Vec<FeeEstimate>> {
        let fee_estimates = self
            .client
            .runtime_api()
            .estimate_fee(block_hash, transactions, simulation_flags)
            .map_err(|e: ApiError| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })???
            .iter()
            .map(|estimate| estimate.into())
            .collect();
        Ok(fee_estimates)
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
        transaction_hash: TransactionHash,
    ) -> RpcApiResult<Vec<starknet_api::transaction::MessageToL1>> {
        self.client.runtime_api().get_tx_messages_to_l1(substrate_block_hash, transaction_hash).map_err(|e| {
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
        let simulations_flags = SimulationFlags { validate: !skip_validate, charge_fee: !skip_fee_charge };
        match tx {
            Transaction::AccountTransaction(tx) => self.simulate_user_tx(block_hash, tx, simulations_flags),
            Transaction::L1HandlerTransaction(tx) => self.simulate_l1_tx(block_hash, tx, simulations_flags),
        }
    }

    fn simulate_user_tx(
        &self,
        block_hash: B::Hash,
        tx: AccountTransaction,
        simulations_flags: SimulationFlags,
    ) -> RpcApiResult<TransactionExecutionInfo> {
        // Simulate a single User Transaction
        // So the result should have single element in vector (index 0)
        let simulation = self
            .client
            .runtime_api()
            .simulate_transactions(block_hash, vec![tx], simulations_flags)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::from(e)
            })?
            .swap_remove(0)
            .map_err(|e| {
                error!("Failed to simulate User Transaction: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })?;
        Ok(simulation.execution_info)
    }

    fn simulate_l1_tx(
        &self,
        block_hash: B::Hash,
        tx: L1HandlerTransaction,
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
                StarknetRpcApiError::from(e)
            })?
            .map_err(|e| {
                error!("Failed to simulate L1 Message: {:?}", e);
                StarknetRpcApiError::InternalServerError
            })
    }
}
