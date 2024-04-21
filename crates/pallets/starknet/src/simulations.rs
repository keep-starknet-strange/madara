use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{Error, SimulationFlags, TransactionSimulationResult};
use mp_transactions::execution::{Execute, ExecutionConfig};
use mp_transactions::{HandleL1MessageTransaction, UserOrL1HandlerTransaction, UserTransaction};
use sp_core::Get;
use sp_runtime::DispatchError;
use starknet_api::transaction::Fee;

use crate::blockifier_state_adapter::{BlockifierStateAdapter, CachedBlockifierStateAdapter};
use crate::execution_config::RuntimeExecutionConfigBuilder;
use crate::{Config, Pallet};

impl<T: Config> Pallet<T> {
    pub fn estimate_fee(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, Error> {
        let mut res = None;

        storage::transactional::with_transaction(|| {
            res = Some(Self::estimate_fee_inner(transactions));
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
        })
        .map_err(|_| Error::FailedToCreateATransactionalStorageExecution)?;

        res.expect("`res` should have been set to `Some` at this point")
    }

    fn estimate_fee_inner(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, Error> {
        let transactions_len = transactions.len();
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config = RuntimeExecutionConfigBuilder::new::<T>().with_query_mode().build();

        let fee_res_iterator = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());

                match Self::execute_transaction_with_state_diff(tx, chain_id, &block_context, &execution_config) {
                    (Ok(execution_info), _) if !execution_info.is_reverted() => Ok(execution_info),
                    (Err(e), _) => {
                        log::error!("Transaction execution failed during fee estimation: {e}");
                        Err(Error::from(e))
                    }
                    (Ok(execution_info), _) => {
                        log::error!(
                            "Transaction execution reverted during fee estimation: {}",
                            // Safe due to the `match` branch order
                            &execution_info.revert_error.clone().unwrap()
                        );
                        Err(Error::TransactionExecutionFailed(execution_info.revert_error.unwrap().to_string()))
                    }
                }
            })
            .map(|exec_info_res| {
                exec_info_res.map(|exec_info| {
                    exec_info
                        .actual_resources
                        .0
                        .get("l1_gas_usage")
                        .ok_or(Error::MissingL1GasUsage)
                        .map(|l1_gas_usage| (exec_info.actual_fee.0 as u64, *l1_gas_usage))
                })
            });

        let mut fees = Vec::with_capacity(transactions_len);
        for fee_res in fee_res_iterator {
            let res = fee_res??;
            fees.push(res);
        }

        Ok(fees)
    }
    pub fn simulate_transactions(
        transactions: Vec<UserTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, Error> {
        let mut res = None;

        storage::transactional::with_transaction(|| {
            res = Some(Self::simulate_transactions_inner(transactions, simulation_flags));
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
        })
        .map_err(|_| Error::FailedToCreateATransactionalStorageExecution)?;

        Ok(res.expect("`res` should have been set to `Some` at this point"))
    }

    fn simulate_transactions_inner(
        transactions: Vec<UserTransaction>,
        simulation_flags: &SimulationFlags,
    ) -> Vec<(CommitmentStateDiff, TransactionSimulationResult)> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config =
            RuntimeExecutionConfigBuilder::new::<T>().with_simulation_mode(simulation_flags).build();

        let tx_execution_results: Vec<(CommitmentStateDiff, TransactionSimulationResult)> = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());

                let res = Self::execute_transaction_with_state_diff(tx, chain_id, &block_context, &execution_config);
                let result = res.0.map_err(|e| {
                    log::error!("Transaction execution failed during simulation: {e}");
                    Error::from(e)
                });
                (res.1, result)
            })
            .collect();

        tx_execution_results
    }

    pub fn simulate_message(
        message: HandleL1MessageTransaction,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<TransactionExecutionInfo, Error>, Error> {
        let mut res = None;

        storage::transactional::with_transaction(|| {
            res = Some(Self::simulate_message_inner(message, simulation_flags));
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
        })
        .map_err(|_| Error::FailedToCreateATransactionalStorageExecution)?;

        Ok(res.expect("`res` should have been set to `Some` at this point"))
    }

    fn simulate_message_inner(
        message: HandleL1MessageTransaction,
        simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, Error> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config =
            RuntimeExecutionConfigBuilder::new::<T>().with_simulation_mode(simulation_flags).build();

        // Follow `offset` from Pallet Starknet where it is set to false
        execution_config.set_offset_version(false);
        let (tx_execution_result, _state_diff) =
            Self::execute_message(message, chain_id, &block_context, &execution_config);

        let tx_execution_result = tx_execution_result.map_err(|e| {
            log::error!("Transaction execution failed during simulation: {e}");
            Error::from(e)
        });

        Self::execute_message(message, chain_id, &block_context, &execution_config).map_err(|e| {
            log::error!("Transaction execution failed during simulation: {e}");
            Error::from(e)
        })
    }

    pub fn estimate_message_fee(message: HandleL1MessageTransaction) -> Result<(u128, u64, u64), Error> {
        let mut res = None;

        storage::transactional::with_transaction(|| {
            res = Some(Self::estimate_message_fee_inner(message));
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
        })
        .map_err(|_| Error::FailedToCreateATransactionalStorageExecution)?;

        res.expect("`res` should have been set to `Some` at this point")
    }

    fn estimate_message_fee_inner(message: HandleL1MessageTransaction) -> Result<(u128, u64, u64), Error> {
        let chain_id = Self::chain_id();

        // Follow `offset` from Pallet Starknet where it is set to false
        let tx_execution_infos =
            match message.into_executable::<T::SystemHash>(chain_id, Fee(u128::MAX), false).execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &Self::get_block_context(),
                &RuntimeExecutionConfigBuilder::new::<T>().with_query_mode().with_disable_nonce_validation().build(),
            ) {
                Ok(execution_info) if !execution_info.is_reverted() => Ok(execution_info),
                Err(e) => {
                    log::error!(
                        "Transaction execution failed during fee estimation: {e} {:?}",
                        std::error::Error::source(&e)
                    );
                    Err(Error::from(e))
                }
                Ok(execution_info) => {
                    let revert_error = execution_info.revert_error;
                    log::error!(
                        "Transaction execution reverted during fee estimation: {}",
                        // Safe due to the `match` branch order
                        &revert_error.clone().unwrap()
                    );
                    Err(Error::TransactionExecutionFailed(revert_error.unwrap().to_string()))
                }
            }?;

        if let Some(l1_gas_usage) = tx_execution_infos.actual_resources.0.get("l1_gas_usage") {
            Ok((T::L1GasPrice::get().price_in_wei, tx_execution_infos.actual_fee.0 as u64, *l1_gas_usage))
        } else {
            Err(Error::MissingL1GasUsage)
        }
    }

    pub fn re_execute_transactions(
        transactions_before: Vec<UserOrL1HandlerTransaction>,
        transactions_to_trace: Vec<UserOrL1HandlerTransaction>,
    ) -> Result<
        Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, Error>,
        Error,
    > {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::re_execute_transactions_inner(
                transactions_before,
                transactions_to_trace,
            )))
        })
        .map_err(|_| Error::FailedToCreateATransactionalStorageExecution)?;

        res.expect("`res` should have been set to `Some` at this point")
    }

    fn re_execute_transactions_inner(
        transactions_before: Vec<UserOrL1HandlerTransaction>,
        transactions_to_trace: Vec<UserOrL1HandlerTransaction>,
    ) -> Result<
        Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, Error>,
        Error,
    > {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let execution_config = RuntimeExecutionConfigBuilder::new::<T>().build();

        Self::execute_user_or_l1_handler_transactions(chain_id, &block_context, &execution_config, transactions_before)
            .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?;
        let transactions_exec_infos = Self::execute_user_or_l1_handler_transactions(
            chain_id,
            &block_context,
            &execution_config,
            transactions_to_trace,
        );

        Ok(transactions_exec_infos)
    }

    fn execute_transaction_with_state_diff(
        transaction: UserTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        execution_config: &ExecutionConfig,
    ) -> (Result<TransactionExecutionInfo, TransactionExecutionError>, CommitmentStateDiff) {
        let mut cached_state = CachedBlockifierStateAdapter(BlockifierStateAdapter::<T>::default());
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

    fn execute_message(
        message: HandleL1MessageTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        execution_config: &ExecutionConfig,
    ) -> (Result<TransactionExecutionInfo, TransactionExecutionError>, CommitmentStateDiff) {
        // Follow `offset` from Pallet Starknet where it is set to false
        let mut cached_state = CachedBlockifierStateAdapter(BlockifierStateAdapter::<T>::default());
        let fee = Fee(u128::MAX);
        let executable = message.into_executable::<T::SystemHash>(chain_id, fee, false);
        let result = executable.execute(&mut cached_state, block_context, execution_config);

        (result, cached_state.to_state_diff())
    }

    fn execute_user_or_l1_handler_transactions(
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        execution_config: &ExecutionConfig,
        transactions: Vec<UserOrL1HandlerTransaction>,
    ) -> Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, PlaceHolderErrorTypeForFailedStarknetExecution>
    {
        let exec_transactions: Vec<_> = transactions
            .iter()
            .map(|user_or_l1_tx| match user_or_l1_tx {
                UserOrL1HandlerTransaction::User(tx) => {
                    Self::execute_transaction_with_state_diff(tx.clone(), chain_id, block_context, execution_config)
                }
                UserOrL1HandlerTransaction::L1Handler(tx, _fee) => {
                    Self::execute_message(tx.clone(), chain_id, block_context, execution_config)
                }
            })
            .collect();

        let mut execution_infos = Vec::with_capacity(exec_transactions.len());
        for (exec_result, state_diff) in exec_transactions {
            match exec_result {
                Ok(info) => execution_infos.push((info, state_diff)),
                Err(err) => {
                    log::error!("Transaction execution failed: {err}");
                    return Err(PlaceHolderErrorTypeForFailedStarknetExecution);
                }
            }
        }

        Ok(execution_infos)
    }
}
