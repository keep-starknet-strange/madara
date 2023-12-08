use alloc::string::String;
use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::CallInfo;
use blockifier::transaction::objects::{TransactionExecutionInfo, TransactionExecutionResult};
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{ExecuteInvocation, RevertedInvocation};
use mp_transactions::execution::Execute;
use mp_transactions::UserTransaction;
use sp_runtime::DispatchError;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::{pallet, Error};

pub fn execute_txs_and_rollback<T: pallet::Config>(
    txs: &Vec<UserTransaction>,
    block_context: &BlockContext,
    chain_id: Felt252Wrapper,
    is_query: bool,
    disable_validation: bool,
    disable_fee_charge: bool,
) -> Result<Vec<TransactionExecutionResult<TransactionExecutionInfo>>, Error<T>> {
    let mut execution_results = vec![];
    storage::transactional::with_transaction(|| {
        for tx in txs {
            let result = match tx {
                UserTransaction::Declare(tx, contract_class) => {
                    let executable =
                        tx.try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), is_query);

                    match executable {
                        Err(err) => Err(err),
                        Ok(executable) => executable.execute(
                            &mut BlockifierStateAdapter::<T>::default(),
                            block_context,
                            is_query,
                            disable_validation,
                            disable_fee_charge,
                        ),
                    }
                }
                UserTransaction::DeployAccount(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, is_query);
                    executable.execute(
                        &mut BlockifierStateAdapter::<T>::default(),
                        block_context,
                        is_query,
                        disable_validation,
                        disable_fee_charge,
                    )
                }
                UserTransaction::Invoke(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, is_query);
                    executable.execute(
                        &mut BlockifierStateAdapter::<T>::default(),
                        block_context,
                        is_query,
                        disable_validation,
                        disable_fee_charge,
                    )
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
