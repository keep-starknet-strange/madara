use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::transaction::objects::TransactionExecutionInfo;
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::PlaceHolderErrorTypeForFailedStarknetExecution;
use mp_transactions::execution::{Execute, ExecutionConfig};
use mp_transactions::UserTransaction;
use sp_runtime::DispatchError;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::{pallet, Error};

pub fn execute_all_txs_and_rollback<T: pallet::Config>(
    txs: &Vec<UserTransaction>,
    block_context: &BlockContext,
    chain_id: Felt252Wrapper,
    execution_config: &mut ExecutionConfig,
) -> Result<Vec<Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>>, Error<T>> {
    let mut execution_results = vec![];

    storage::transactional::with_transaction(|| {
        for tx in txs {
            execution_config.set_offset_version(tx.offset_version());
            let result = match tx {
                UserTransaction::Declare(tx, contract_class) => tx
                    .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                    .and_then(|exec| {
                        exec.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                    }),
                UserTransaction::DeployAccount(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                }
                UserTransaction::Invoke(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                }
            }
            .map_err(|e| {
                log::info!("Failed to execute transaction: {:?}", e);
                PlaceHolderErrorTypeForFailedStarknetExecution
            });

            execution_results.push(result);
        }
        storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
    })
    .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?;

    Ok(execution_results)
}

pub fn execute_txs_until_first_failed_and_rollback<T: pallet::Config>(
    txs: &Vec<UserTransaction>,
    block_context: &BlockContext,
    chain_id: Felt252Wrapper,
    execution_config: &mut ExecutionConfig,
) -> Result<Result<Vec<TransactionExecutionInfo>, PlaceHolderErrorTypeForFailedStarknetExecution>, Error<T>> {
    let mut execution_infos = vec![];

    storage::transactional::with_transaction(|| {
        for tx in txs {
            execution_config.set_offset_version(tx.offset_version());
            let result = match tx {
                UserTransaction::Declare(tx, contract_class) => tx
                    .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                    .and_then(|exec| {
                        exec.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                    }),
                UserTransaction::DeployAccount(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                }
                UserTransaction::Invoke(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    executable.execute(&mut BlockifierStateAdapter::<T>::default(), block_context, execution_config)
                }
            };

            // In case of revert (cairo1) or failed assertion (cairo0) we stop iterating
            // For now we don't know how to move the revert_reason out of the runtime, so we can keep things
            // simple with the error handling
            match result {
                Ok(tx_info) if !tx_info.is_reverted() => execution_infos.push(tx_info),
                _ => break,
            };
        }
        storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
    })
    .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?;

    // All transation were successfully executed
    Ok(if execution_infos.len() == txs.len() {
        Ok(execution_infos)
    } else {
        Err(PlaceHolderErrorTypeForFailedStarknetExecution)
    })
}
