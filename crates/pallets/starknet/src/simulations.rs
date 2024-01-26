use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::state::cached_state::{CachedState, CommitmentStateDiff};
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags};
use mp_transactions::execution::{Execute, ExecutionConfig};
use mp_transactions::{HandleL1MessageTransaction, UserTransaction};
use sp_core::Get;
use sp_runtime::DispatchError;
use starknet_api::transaction::Fee;

use crate::blockifier_state_adapter::{BlockifierStateAdapter, CachedBlockifierStateAdapter};
use crate::execution_config::RuntimeExecutionConfigBuilder;
use crate::types::TransactionSimulationResult;
use crate::{Config, Error, Pallet};

impl<T: Config> Pallet<T> {
    pub fn estimate_fee(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_fee_inner(
                transactions,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn estimate_fee_inner(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, DispatchError> {
        let transactions_len = transactions.len();
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config = RuntimeExecutionConfigBuilder::new::<T>().with_query_mode().build();

        let fee_res_iterator = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());

                match Self::execute_transaction(tx, chain_id, &block_context, &execution_config) {
                    Ok(execution_info) if !execution_info.is_reverted() => Ok(execution_info),
                    Err(e) => {
                        log::error!("Transaction execution failed during fee estimation: {e}");
                        Err(Error::<T>::TransactionExecutionFailed)
                    }
                    Ok(execution_info) => {
                        log::error!(
                            "Transaction execution reverted during fee estimation: {}",
                            // Safe due to the `match` branch order
                            execution_info.revert_error.unwrap()
                        );
                        Err(Error::<T>::TransactionExecutionFailed)
                    }
                }
            })
            .map(|exec_info_res| {
                exec_info_res.map(|exec_info| {
                    exec_info
                        .actual_resources
                        .0
                        .get("l1_gas_usage")
                        .ok_or_else(|| DispatchError::from(Error::<T>::MissingL1GasUsage))
                        .map(|l1_gas_usage| (exec_info.actual_fee.0 as u64, *l1_gas_usage))
                })
            });

        let mut fees = Vec::with_capacity(transactions_len);
        for fee_res in fee_res_iterator {
            fees.push(fee_res??);
        }

        Ok(fees)
    }
    pub fn simulate_transactions(
        transactions: Vec<UserTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<(TransactionSimulationResult, CommitmentStateDiff)>, DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::simulate_transactions_inner(
                transactions,
                simulation_flags,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn simulate_transactions_inner(
        transactions: Vec<UserTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<(TransactionSimulationResult, CommitmentStateDiff)>, DispatchError> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config =
            RuntimeExecutionConfigBuilder::new::<T>().with_simulation_mode(simulation_flags).build();

        let tx_execution_results = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());

                let res = Self::execute_transaction_with_state_diff(tx, chain_id, &block_context, &execution_config);
                let result = res.0.map_err(|e| {
                    log::error!("Transaction execution failed during simulation: {e}");
                    PlaceHolderErrorTypeForFailedStarknetExecution
                });
                (result, res.1)
            })
            .collect();

        Ok(tx_execution_results)
    }

    pub fn estimate_message_fee(message: HandleL1MessageTransaction) -> Result<(u128, u64, u64), DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_message_fee_inner(
                message,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn estimate_message_fee_inner(message: HandleL1MessageTransaction) -> Result<(u128, u64, u64), DispatchError> {
        let chain_id = Self::chain_id();

        let tx_execution_infos = message
            .into_executable::<T::SystemHash>(chain_id, Fee::default(), true)
            .execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &Self::get_block_context(),
                &RuntimeExecutionConfigBuilder::new::<T>().with_query_mode().with_disable_nonce_validation().build(),
            )
            .map_err(|e| {
                log::error!("L1 message execution failed during fee estimation: {}", e);
                Error::<T>::TransactionExecutionFailed
            })?;

        if let Some(l1_gas_usage) = tx_execution_infos.actual_resources.0.get("l1_gas_usage") {
            Ok((T::L1GasPrice::get().price_in_wei, tx_execution_infos.actual_fee.0 as u64, *l1_gas_usage))
        } else {
            Err(Error::<T>::MissingL1GasUsage.into())
        }
    }

    fn execute_transaction(
        transaction: UserTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        execution_config: &ExecutionConfig,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        match transaction {
            UserTransaction::Declare(tx, contract_class) => {
                tx.try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version()).and_then(
                    |exec| exec.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config),
                )
            }
            UserTransaction::DeployAccount(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
            }
            UserTransaction::Invoke(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
            }
        }
    }

    fn execute_transaction_with_state_diff(
        transaction: UserTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        execution_config: &ExecutionConfig,
    ) -> (Result<TransactionExecutionInfo, TransactionExecutionError>, CommitmentStateDiff) {
        let mut cached_state = CachedBlockifierStateAdapter(CachedState::from(BlockifierStateAdapter::<T>::default()));
        let result = match transaction {
            UserTransaction::Declare(tx, contract_class) => tx
                .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                .and_then(|exec| exec.execute(&mut cached_state, block_context, execution_config)),
            UserTransaction::DeployAccount(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                executable.execute(&mut cached_state, block_context, execution_config)
            }
            UserTransaction::Invoke(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                executable.execute(&mut cached_state, block_context, execution_config)
            }
        };

        (result, cached_state.to_state_diff())
    }
}
