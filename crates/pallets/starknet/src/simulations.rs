use blockifier::context::BlockContext;
use blockifier::state::cached_state::{CachedState, CommitmentStateDiff, GlobalContractCache};
use blockifier::state::state_api::State;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::{ExecutableTransaction, L1HandlerTransaction};
use frame_support::storage;
use mp_simulations::{InternalSubstrateError, SimulationError, SimulationFlags, TransactionSimulationResult};
use mp_transactions::execution::{
    commit_transactional_state, execute_l1_handler_transaction, run_non_revertible_transaction,
    run_revertible_transaction, MutRefState, SetArbitraryNonce,
};
use sp_core::Get;
use sp_runtime::DispatchError;
use starknet_api::transaction::TransactionVersion;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::{log, Config, Error, Pallet};

type ReExecutionResult = Result<Vec<(TransactionExecutionInfo, Option<CommitmentStateDiff>)>, SimulationError>;

impl<T: Config> Pallet<T> {
    pub fn estimate_fee(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<Vec<(u128, u128)>, SimulationError>, InternalSubstrateError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_fee_inner(
                transactions,
                simulation_flags,
            )))
        })
        .map_err(|e| {
            log::error!("Transaction execution failed during estimate_fee: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })
    }

    fn estimate_fee_inner(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<(u128, u128)>, SimulationError> {
        let transactions_len = transactions.len();
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        let fee_res_iterator = transactions
            .into_iter()
            .map(|tx| match Self::execute_account_transaction(&tx, &mut state, &block_context, simulation_flags) {
                Ok(execution_info) => {
                    if !execution_info.is_reverted() {
                        Ok(execution_info)
                    } else {
                        log!(
                            debug,
                            "Transaction execution reverted during fee estimation: {:?}",
                            execution_info.revert_error
                        );
                        Err(SimulationError::TransactionExecutionFailed(
                            execution_info.revert_error.unwrap().to_string(),
                        ))
                    }
                }
                Err(e) => {
                    log!(debug, "Transaction execution failed during fee estimation: {:?}", e);
                    Err(SimulationError::from(e))
                }
            })
            .map(|exec_info_res| {
                exec_info_res.map(|exec_info| {
                    exec_info
                        .actual_resources
                        .0
                        .get("l1_gas_usage")
                        .ok_or_else(|| DispatchError::from(Error::<T>::MissingL1GasUsage))
                        .map(|l1_gas_usage| (exec_info.actual_fee.0, *l1_gas_usage))
                })
            });

        let mut fees = Vec::with_capacity(transactions_len);
        for fee_res in fee_res_iterator {
            let res = fee_res?.map_err(|_| SimulationError::StateDiff)?;
            fees.push(res);
        }

        Ok(fees)
    }

    pub fn simulate_transactions(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, SimulationError>, InternalSubstrateError>
    {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::simulate_transactions_inner(
                transactions,
                simulation_flags,
            )))
        })
        .map_err(|e| {
            log::error!("Transaction Simulation failed during simulate_transaction: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })
    }
    fn simulate_transactions_inner(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, SimulationError> {
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        let tx_execution_results: Vec<(CommitmentStateDiff, TransactionSimulationResult)> = transactions
            .into_iter()
            .map(|tx| {
                let res = Self::execute_account_transaction_with_state_diff(
                    &tx,
                    &mut state,
                    &block_context,
                    simulation_flags,
                )?;

                let result = res.0.map_err(|e| {
                    log::error!("Transaction execution failed during simulation: {e}");
                    SimulationError::from(e)
                });

                Ok((res.1, result))
            })
            .collect::<Result<Vec<_>, SimulationError>>()?;

        Ok(tx_execution_results)
    }

    pub fn simulate_message(
        message: L1HandlerTransaction,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<TransactionExecutionInfo, SimulationError>, InternalSubstrateError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::simulate_message_inner(
                message,
                simulation_flags,
            )))
        })
        .map_err(|e| {
            log::error!("Transaction Simulation failed during simulate_message: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })
    }

    fn simulate_message_inner(
        message: L1HandlerTransaction,
        _simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, SimulationError> {
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        Self::execute_message(&message, &mut state, &block_context).map_err(|e| {
            log::error!("Transaction execution failed during simulation: {e}");
            SimulationError::from(e)
        })
    }

    pub fn estimate_message_fee(
        message: L1HandlerTransaction,
    ) -> Result<Result<(u128, u128, u128), SimulationError>, InternalSubstrateError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_message_fee_inner(
                message,
            )))
        })
        .map_err(|e| {
            log::error!("Transaction Simulation failed during estimate_message_fee: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })
    }

    fn estimate_message_fee_inner(message: L1HandlerTransaction) -> Result<(u128, u128, u128), SimulationError> {
        let mut cached_state = Self::init_cached_state();

        let tx_execution_infos = match message.execute(&mut cached_state, &Self::get_block_context(), true, true) {
            Ok(execution_info) if !execution_info.is_reverted() => Ok(execution_info),
            Err(e) => {
                log!(
                    debug,
                    "Transaction execution failed during fee estimation: {:?} {:?}",
                    e,
                    std::error::Error::source(&e)
                );
                Err(SimulationError::from(e))
            }
            Ok(execution_info) => {
                log!(
                    debug,
                    "Transaction execution reverted during fee estimation: {}",
                    // Safe due to the `match` branch order
                    &execution_info.revert_error.clone().unwrap()
                );
                Err(SimulationError::TransactionExecutionFailed(execution_info.revert_error.unwrap().to_string()))
            }
        }?;

        if let Some(l1_gas_usage) = tx_execution_infos.actual_resources.0.get("l1_gas_usage") {
            Ok((T::L1GasPrices::get().eth_l1_gas_price.into(), tx_execution_infos.actual_fee.0 as u128, *l1_gas_usage))
        } else {
            Err(SimulationError::MissingL1GasUsage)
        }
    }

    pub fn re_execute_transactions(
        transactions_before: Vec<Transaction>,
        transactions_to_trace: Vec<Transaction>,
        with_state_diff: bool,
    ) -> Result<ReExecutionResult, InternalSubstrateError> {
        storage::transactional::with_transaction(|| {
            let res = Self::re_execute_transactions_inner(transactions_before, transactions_to_trace, with_state_diff);
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Ok(res)))
        })
        .map_err(|e| {
            log::error!("Failed to reexecute a tx: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })?
    }

    fn re_execute_transactions_inner(
        transactions_before: Vec<Transaction>,
        transactions_to_trace: Vec<Transaction>,
        with_state_diff: bool,
    ) -> Result<Vec<(TransactionExecutionInfo, Option<CommitmentStateDiff>)>, SimulationError> {
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        transactions_before.iter().try_for_each(|tx| {
            Self::execute_transaction(tx, &mut state, &block_context, &SimulationFlags::default()).map_err(|e| {
                log::error!("Failed to reexecute a tx: {}", e);
                SimulationError::from(e)
            })?;
            Ok::<(), SimulationError>(())
        })?;

        let execution_infos = transactions_to_trace
            .iter()
            .map(|tx| {
                let mut transactional_state =
                    CachedState::new(MutRefState::new(&mut state), GlobalContractCache::new(1));
                let res = Self::execute_transaction(
                    tx,
                    &mut transactional_state,
                    &block_context,
                    &SimulationFlags::default(),
                )
                .map_err(|e| {
                    log::error!("Failed to reexecute a tx: {}", e);
                    SimulationError::from(e)
                });

                let res = res
                    .map(|r| if with_state_diff { (r, Some(transactional_state.to_state_diff())) } else { (r, None) });
                commit_transactional_state(transactional_state).map_err(|e| {
                    log::error!("Failed to commit state changes: {:?}", e);
                    SimulationError::from(e)
                })?;

                res
            })
            .collect::<Result<_, SimulationError>>()?;

        Ok(execution_infos)
    }

    fn execute_transaction<S: State + SetArbitraryNonce>(
        transaction: &Transaction,
        state: &mut S,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        match transaction {
            Transaction::AccountTransaction(tx) => {
                Self::execute_account_transaction(tx, state, block_context, simulation_flags)
            }

            Transaction::L1HandlerTransaction(tx) => Self::execute_message(tx, state, block_context),
        }
    }

    fn execute_account_transaction<S: State + SetArbitraryNonce>(
        transaction: &AccountTransaction,
        state: &mut S,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        match transaction {
            AccountTransaction::Declare(tx) => run_non_revertible_transaction(
                tx,
                state,
                block_context,
                simulation_flags.validate,
                simulation_flags.charge_fee,
            ),
            AccountTransaction::DeployAccount(tx) => run_non_revertible_transaction(
                tx,
                state,
                block_context,
                simulation_flags.validate,
                simulation_flags.charge_fee,
            ),
            AccountTransaction::Invoke(tx) if tx.tx.version() == TransactionVersion::ZERO => {
                run_non_revertible_transaction(
                    tx,
                    state,
                    block_context,
                    simulation_flags.validate,
                    simulation_flags.charge_fee,
                )
            }
            AccountTransaction::Invoke(tx) => run_revertible_transaction(
                tx,
                state,
                block_context,
                simulation_flags.validate,
                simulation_flags.charge_fee,
            ),
        }
    }

    fn execute_account_transaction_with_state_diff<S: State + SetArbitraryNonce>(
        transaction: &AccountTransaction,
        state: &mut S,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> Result<(Result<TransactionExecutionInfo, TransactionExecutionError>, CommitmentStateDiff), SimulationError>
    {
        // In order to produce a state diff for this specific tx we execute on a transactional state
        let mut transactional_state = CachedState::new(MutRefState::new(state), GlobalContractCache::new(1));

        let result =
            Self::execute_account_transaction(transaction, &mut transactional_state, block_context, simulation_flags);

        let state_diff = transactional_state.to_state_diff();
        // Once the state diff of this tx is generated, we apply those changes on the original state
        // so that next txs being simulated are ontop of this one (avoid nonce error)
        commit_transactional_state(transactional_state).map_err(|e| {
            log::error!("Failed to commit state changes: {:?}", e);
            SimulationError::from(e)
        })?;

        Ok((result, state_diff))
    }

    fn execute_message<S: State>(
        transaction: &L1HandlerTransaction,
        state: &mut S,
        block_context: &BlockContext,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        execute_l1_handler_transaction(transaction, state, block_context)
    }
}
