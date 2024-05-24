use blockifier::blockifier::block::GasPrices;
use blockifier::context::BlockContext;
use blockifier::state::cached_state::{CachedState, CommitmentStateDiff, GlobalContractCache};
use blockifier::state::state_api::State;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::{FeeType, GasVector, HasRelatedFeeType, TransactionExecutionInfo};
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::{ExecutableTransaction, L1HandlerTransaction};
use frame_support::storage;
use mp_simulations::{
    FeeEstimate, InternalSubstrateError, ReExecutionResult, SimulationError, SimulationFlags, TransactionSimulation,
    TransactionSimulationResult,
};
use mp_transactions::execution::{
    commit_transactional_state, execute_l1_handler_transaction, run_non_revertible_transaction,
    run_revertible_transaction, CheckFeeBounds, MutRefState, SetArbitraryNonce,
};
use sp_runtime::DispatchError;
use starknet_api::transaction::TransactionVersion;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::{log, Config, Pallet};

impl<T: Config> Pallet<T> {
    pub fn estimate_fee(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<Vec<FeeEstimate>, SimulationError>, InternalSubstrateError> {
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
    ) -> Result<Vec<FeeEstimate>, SimulationError> {
        let transactions_len = transactions.len();
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        let fee_res_iterator = transactions.into_iter().map(|tx| {
            match Self::execute_account_transaction(&tx, &mut state, &block_context, simulation_flags) {
                Ok(mut execution_info) => {
                    if !execution_info.is_reverted() {
                        Self::execution_info_to_fee_estimate(&tx, &mut execution_info, &block_context)
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
            }
        });

        let mut fees = Vec::with_capacity(transactions_len);
        for fee_res in fee_res_iterator {
            let res = fee_res?;
            fees.push(res);
        }

        Ok(fees)
    }

    pub fn simulate_transactions(
        transactions: Vec<AccountTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<TransactionSimulationResult>, InternalSubstrateError> {
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
    ) -> Vec<TransactionSimulationResult> {
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        let tx_execution_results = transactions
            .into_iter()
            .map(|tx| {
                // In order to produce a state diff for this specific tx we execute on a transactional state
                let mut transactional_state =
                    CachedState::new(MutRefState::new(&mut state), GlobalContractCache::new(1));

                let exec_info =
                    Self::execute_account_transaction(&tx, &mut transactional_state, &block_context, simulation_flags)
                        .map_err(|e| {
                            log!(debug, "Failed to execute transaction: {:?}", e);
                            SimulationError::from(e)
                        });

                let mut exec_info = match exec_info {
                    Ok(exec_info) => exec_info,
                    Err(e) => return Err(e),
                };

                let state_diff = transactional_state.to_state_diff();
                // Once the state diff of this tx is generated, we apply those changes on the original state
                // so that next txs being simulated are ontop of this one (avoid nonce error)
                match commit_transactional_state(transactional_state) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Failed to commit state changes: {:?}", e);
                        return Err(SimulationError::from(e));
                    }
                };

                let fee_estimate = match Self::execution_info_to_fee_estimate(&tx, &mut exec_info, &block_context) {
                    Ok(fee_estimate) => fee_estimate,
                    Err(e) => return Err(e),
                };
                Ok(TransactionSimulation { fee_estimate, execution_info: exec_info, state_diff })
            })
            .collect();

        tx_execution_results
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
    ) -> Result<Result<FeeEstimate, SimulationError>, InternalSubstrateError> {
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

    fn estimate_message_fee_inner(message: L1HandlerTransaction) -> Result<FeeEstimate, SimulationError> {
        let mut cached_state = Self::init_cached_state();
        let fee_type = message.fee_type();

        let mut tx_execution_info = match message.execute(&mut cached_state, &Self::get_block_context(), true, true) {
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

        let current_l1_gas_price: GasPrices = Self::current_l1_gas_prices().into();
        Self::from_tx_info_and_gas_price(
            &mut tx_execution_info,
            &current_l1_gas_price,
            fee_type,
            None,
            &Self::get_block_context(),
        )
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

        let simulation_flags =
            SimulationFlags { charge_fee: !Self::is_transaction_fee_disabled(), ..Default::default() };
        let execution_infos = transactions_to_trace
            .iter()
            .map(|tx| {
                let mut transactional_state =
                    CachedState::new(MutRefState::new(&mut state), GlobalContractCache::new(1));
                let res = Self::execute_transaction(tx, &mut transactional_state, &block_context, &simulation_flags)
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

    pub fn get_transaction_re_execution_state_diff(
        transactions_before: Vec<Transaction>,
        transactions_to_trace: Vec<Transaction>,
    ) -> Result<Result<CommitmentStateDiff, SimulationError>, InternalSubstrateError> {
        storage::transactional::with_transaction(|| {
            let res = Self::get_transaction_re_execution_state_diff_inner(transactions_before, transactions_to_trace);
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Ok(res)))
        })
        .map_err(|e| {
            log::error!("Failed to reexecute a tx: {:?}", e);
            InternalSubstrateError::FailedToCreateATransactionalStorageExecution
        })?
    }

    fn get_transaction_re_execution_state_diff_inner(
        transactions_before: Vec<Transaction>,
        transactions_to_trace: Vec<Transaction>,
    ) -> Result<CommitmentStateDiff, SimulationError> {
        let block_context = Self::get_block_context();
        let mut state = BlockifierStateAdapter::<T>::default();

        transactions_before.iter().try_for_each(|tx| {
            Self::execute_transaction(tx, &mut state, &block_context, &SimulationFlags::default()).map_err(|e| {
                log::error!("Failed to reexecute a tx: {}", e);
                SimulationError::from(e)
            })?;
            Ok::<(), SimulationError>(())
        })?;

        let mut transactional_state = CachedState::new(MutRefState::new(&mut state), GlobalContractCache::new(1));

        transactions_to_trace.iter().try_for_each(|tx| {
            Self::execute_transaction(tx, &mut transactional_state, &block_context, &SimulationFlags::default())
                .map_err(|e| {
                    log::error!("Failed to reexecute a tx: {}", e);
                    SimulationError::from(e)
                })?;
            Ok::<(), SimulationError>(())
        })?;

        let state_diff = transactional_state.to_state_diff();
        commit_transactional_state(transactional_state).map_err(|e| {
            log::error!("Failed to commit state changes: {:?}", e);
            SimulationError::from(e)
        })?;

        Ok(state_diff)
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

    fn execution_info_to_fee_estimate(
        transaction: &AccountTransaction,
        execution_info: &mut TransactionExecutionInfo,
        block_context: &BlockContext,
    ) -> Result<FeeEstimate, SimulationError> {
        let tx_context = block_context.to_tx_context(transaction);
        let gas_vector = match transaction.clone() {
            AccountTransaction::Declare(tx) => tx.estimate_minimal_gas_vector(&tx_context)?,
            AccountTransaction::DeployAccount(tx) => tx.estimate_minimal_gas_vector(&tx_context)?,
            AccountTransaction::Invoke(tx) => tx.estimate_minimal_gas_vector(&tx_context)?,
        };
        let current_l1_gas_price: GasPrices = Self::current_l1_gas_prices().into();
        Self::from_tx_info_and_gas_price(
            execution_info,
            &current_l1_gas_price,
            transaction.fee_type(),
            Some(gas_vector),
            block_context,
        )
    }

    fn execute_message<S: State>(
        transaction: &L1HandlerTransaction,
        state: &mut S,
        block_context: &BlockContext,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        execute_l1_handler_transaction(transaction, state, block_context)
    }
}

// Took inspiration from here - https://github.com/eqlabs/pathfinder/blob/4a18125cae2c8fb1284e9e8fd23acf5d5bcfde18/crates/executor/src/types.rs#L41-L41
impl<T: Config> Pallet<T> {
    /// Computes fee estimate from the transaction execution information.
    ///
    /// `TransactionExecutionInfo` contains two related fields:
    /// - `TransactionExecutionInfo::actual_fee` is the overall cost of the transaction (in WEI/FRI)
    /// - `TransactionExecutionInfo::da_gas` is the gas usage for _data availability_.
    ///
    /// The problem is that we have to return both `gas_usage` and
    /// `data_gas_usage` but we don't directly have the value of `gas_usage`
    /// from the execution info, so we have to calculate that from other
    /// fields.
    fn from_tx_info_and_gas_price(
        tx_info: &mut TransactionExecutionInfo,
        gas_prices: &GasPrices,
        fee_type: FeeType,
        minimal_l1_gas_amount_vector: Option<GasVector>,
        block_context: &BlockContext,
    ) -> Result<FeeEstimate, SimulationError> {
        let gas_price = gas_prices.get_gas_price_by_fee_type(&fee_type).get();
        let data_gas_price = gas_prices.get_data_gas_price_by_fee_type(&fee_type).get();
        if tx_info.actual_fee.0 == 0 {
            // fee is not calculated by default for L1 handler transactions and if max_fee
            // is zero, we have to do that explicitly
            tx_info.actual_fee =
                match blockifier::fee::fee_utils::calculate_tx_fee(&tx_info.actual_resources, block_context, &fee_type)
                {
                    Ok(fee) => fee,
                    Err(e) => {
                        log!(debug, "Failed to calculate tx fee: {:?}", e);
                        return Err(SimulationError::from(e));
                    }
                };
        }
        let data_gas_consumed = tx_info.da_gas.l1_data_gas;
        let data_gas_fee = data_gas_consumed.saturating_mul(data_gas_price);
        let gas_consumed = tx_info.actual_fee.0.saturating_sub(data_gas_fee) / gas_price.max(1);

        let (minimal_gas_consumed, minimal_data_gas_consumed) =
            minimal_l1_gas_amount_vector.map(|v| (v.l1_gas, v.l1_data_gas)).unwrap_or_default();

        let gas_consumed = gas_consumed.max(minimal_gas_consumed);
        let data_gas_consumed = data_gas_consumed.max(minimal_data_gas_consumed);
        let overall_fee =
            gas_consumed.saturating_mul(gas_price).saturating_add(data_gas_consumed.saturating_mul(data_gas_price));

        Ok(FeeEstimate { gas_consumed, gas_price, data_gas_consumed, data_gas_price, overall_fee, fee_type })
    }
}
