use alloc::vec::Vec;

use blockifier::context::BlockContext;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::transactions::ExecutableTransaction;
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags, TransactionSimulationResult};
use mp_transactions::{HandleL1MessageTransaction, UserOrL1HandlerTransaction, UserTransaction};
use sp_core::Get;
use sp_runtime::DispatchError;
use starknet_api::transaction::Fee;

use crate::{Config, Error, Pallet};

impl<T: Config> Pallet<T> {
    pub fn estimate_fee(transactions: Vec<UserTransaction>) -> Result<Vec<(u128, u128)>, DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_fee_inner(
                transactions,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn estimate_fee_inner(transactions: Vec<UserTransaction>) -> Result<Vec<(u128, u128)>, DispatchError> {
        let transactions_len = transactions.len();
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();

        let fee_res_iterator = transactions
            .into_iter()
            .map(|tx| {
                match Self::execute_user_transaction(tx, chain_id, &block_context, &SimulationFlags::default()) {
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
                        .map(|l1_gas_usage| (exec_info.actual_fee.0, *l1_gas_usage))
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
    ) -> Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, DispatchError> {
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
    ) -> Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, DispatchError> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();

        let tx_execution_results: Vec<(CommitmentStateDiff, TransactionSimulationResult)> = transactions
            .into_iter()
            .map(|tx| {
                let res = Self::execute_transaction_with_state_diff(tx, chain_id, &block_context, simulation_flags);
                let result = res.0.map_err(|e| {
                    log::error!("Transaction execution failed during simulation: {e}");
                    PlaceHolderErrorTypeForFailedStarknetExecution
                });
                (res.1, result)
            })
            .collect();

        Ok(tx_execution_results)
    }

    pub fn simulate_message(
        message: HandleL1MessageTransaction,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>, DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::simulate_message_inner(
                message,
                simulation_flags,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn simulate_message_inner(
        message: HandleL1MessageTransaction,
        simulation_flags: &SimulationFlags,
    ) -> Result<Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>, DispatchError> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();

        let tx_execution_result =
            Self::execute_message(message, chain_id, Fee(u128::MAX), &block_context, simulation_flags).map_err(|e| {
                log::error!("Transaction execution failed during simulation: {e}");
                PlaceHolderErrorTypeForFailedStarknetExecution
            });

        Ok(tx_execution_result)
    }

    pub fn estimate_message_fee(message: HandleL1MessageTransaction) -> Result<(u128, u128, u128), DispatchError> {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::estimate_message_fee_inner(
                message,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn estimate_message_fee_inner(message: HandleL1MessageTransaction) -> Result<(u128, u128, u128), DispatchError> {
        let chain_id = Self::chain_id();
        let mut cached_state = Self::init_cached_state();

        let tx_execution_infos = match message.into_executable::<T::SystemHash>(chain_id, Fee(u128::MAX), true).execute(
            &mut cached_state,
            &Self::get_block_context(),
            true,
            true,
        ) {
            Ok(execution_info) if !execution_info.is_reverted() => Ok(execution_info),
            Err(e) => {
                log::error!(
                    "Transaction execution failed during fee estimation: {e} {:?}",
                    std::error::Error::source(&e)
                );
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
        }?;

        if let Some(l1_gas_usage) = tx_execution_infos.actual_resources.0.get("l1_gas_usage") {
            Ok((T::L1GasPrices::get().eth_l1_gas_price.into(), tx_execution_infos.actual_fee.0 as u128, *l1_gas_usage))
        } else {
            Err(Error::<T>::MissingL1GasUsage.into())
        }
    }

    pub fn re_execute_transactions(
        transactions: Vec<UserOrL1HandlerTransaction>,
    ) -> Result<
        Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, PlaceHolderErrorTypeForFailedStarknetExecution>,
        DispatchError,
    > {
        storage::transactional::with_transaction(|| {
            storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(Self::re_execute_transactions_inner(
                transactions,
            )))
        })
        .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?
    }

    fn re_execute_transactions_inner(
        transactions: Vec<UserOrL1HandlerTransaction>,
    ) -> Result<
        Result<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>, PlaceHolderErrorTypeForFailedStarknetExecution>,
        DispatchError,
    > {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();

        let execution_infos = transactions
            .into_iter()
            .map(|user_or_l1_tx| {
                let mut cached_state = Self::init_cached_state();

                let res = match user_or_l1_tx {
                    UserOrL1HandlerTransaction::User(tx) => {
                        Self::execute_user_transaction(tx, chain_id, &block_context, &SimulationFlags::default())
                            .map_err(|e| {
                                log::error!("Failed to reexecute a tx: {}", e);
                                PlaceHolderErrorTypeForFailedStarknetExecution
                            })
                    }
                    UserOrL1HandlerTransaction::L1Handler(tx, fee) => {
                        Self::execute_message(tx, chain_id, fee, &block_context, &SimulationFlags::default()).map_err(
                            |e| {
                                log::error!("Failed to reexecute a tx: {}", e);
                                PlaceHolderErrorTypeForFailedStarknetExecution
                            },
                        )
                    }
                };

                res.map(|r| (r, cached_state.to_state_diff()))
            })
            .collect();

        Ok(execution_infos)
    }

    fn execute_user_transaction(
        transaction: UserTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        let mut cached_state = Self::init_cached_state();

        match transaction {
            UserTransaction::Declare(tx, contract_class) => tx
                .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                .and_then(|exec| {
                    AccountTransaction::Declare(exec).execute(
                        &mut cached_state,
                        block_context,
                        simulation_flags.charge_fee,
                        simulation_flags.validate,
                    )
                }),
            UserTransaction::DeployAccount(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                AccountTransaction::DeployAccount(executable).execute(
                    &mut cached_state,
                    block_context,
                    simulation_flags.charge_fee,
                    simulation_flags.validate,
                )
            }
            UserTransaction::Invoke(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                AccountTransaction::Invoke(executable).execute(
                    &mut cached_state,
                    block_context,
                    simulation_flags.charge_fee,
                    simulation_flags.validate,
                )
            }
        }
    }

    fn execute_transaction_with_state_diff(
        transaction: UserTransaction,
        chain_id: Felt252Wrapper,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> (Result<TransactionExecutionInfo, TransactionExecutionError>, CommitmentStateDiff) {
        let mut cached_state = Self::init_cached_state();

        let result = match transaction {
            UserTransaction::Declare(tx, contract_class) => tx
                .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                .and_then(|exec| {
                    AccountTransaction::Declare(exec).execute(
                        &mut cached_state,
                        block_context,
                        simulation_flags.charge_fee,
                        simulation_flags.validate,
                    )
                }),
            UserTransaction::DeployAccount(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                AccountTransaction::DeployAccount(executable).execute(
                    &mut cached_state,
                    block_context,
                    simulation_flags.charge_fee,
                    simulation_flags.validate,
                )
            }
            UserTransaction::Invoke(tx) => {
                let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                AccountTransaction::Invoke(executable).execute(
                    &mut cached_state,
                    block_context,
                    simulation_flags.charge_fee,
                    simulation_flags.validate,
                )
            }
        };

        (result, cached_state.to_state_diff())
    }

    fn execute_message(
        message: HandleL1MessageTransaction,
        chain_id: Felt252Wrapper,
        fee: Fee,
        block_context: &BlockContext,
        simulation_flags: &SimulationFlags,
    ) -> Result<TransactionExecutionInfo, TransactionExecutionError> {
        let mut cached_state = Self::init_cached_state();

        // Follow `offset` from Pallet Starknet where it is set to false
        let executable = message.into_executable::<T::SystemHash>(chain_id, fee, false);
        executable.execute(&mut cached_state, block_context, simulation_flags.charge_fee, simulation_flags.validate)
    }
}
