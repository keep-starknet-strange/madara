use alloc::string::String;
use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::CallInfo;
use blockifier::transaction::objects::{TransactionExecutionInfo, TransactionExecutionResult};
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{ExecuteInvocation, RevertedInvocation};
use mp_transactions::execution::{Execute, ExecutionConfig};
use mp_transactions::UserTransaction;
use sp_runtime::DispatchError;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::{pallet, Error};

pub fn execute_txs_and_rollback<T: pallet::Config>(
    txs: &Vec<UserTransaction>,
    block_context: &BlockContext,
    chain_id: Felt252Wrapper,
    execution_config: &mut ExecutionConfig,
) -> Result<Vec<TransactionExecutionResult<TransactionExecutionInfo>>, Error<T>> {
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
            };
            execution_results.push(result);
        }
        storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
    })
    .map_err(|_| Error::<T>::TransactionalExecutionFailed)?;
    Ok(execution_results)
}

pub fn convert_call_info_to_execute_invocation<T>(
    call_info: &CallInfo,
    revert_error: Option<&String>,
) -> Result<ExecuteInvocation, Error<T>> {
    if call_info.execution.failed {
        return Ok(ExecuteInvocation::Reverted(RevertedInvocation {
            revert_reason: revert_error.ok_or(Error::MissingRevertReason)?.clone(),
        }));
    }
    Ok(ExecuteInvocation::Success(call_info.try_into().map_err(|_| Error::TransactionExecutionFailed)?))
}
