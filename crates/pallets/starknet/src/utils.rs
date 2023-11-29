use alloc::string::String;
use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::CallInfo;
use blockifier::transaction::objects::{TransactionExecutionInfo, TransactionExecutionResult};
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::{ExecuteInvocation, FunctionInvocation, RevertedInvocation};
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
) -> Vec<TransactionExecutionResult<TransactionExecutionInfo>> {
    let mut execution_results = vec![];
    let _: Result<_, DispatchError> = storage::transactional::with_transaction(|| {
        for tx in txs {
            let result = match tx {
                UserTransaction::Declare(tx, contract_class) => {
                    let executable = tx
                        .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), is_query)
                        .map_err(|_| Error::<T>::InvalidContractClass)
                        .expect("Contract class should be valid");
                    executable.execute(
                        &mut BlockifierStateAdapter::<T>::default(),
                        block_context,
                        is_query,
                        disable_validation,
                        disable_fee_charge,
                    )
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
            log::debug!("Executed transaction in rollback mode: {:#?}", result);
            execution_results.push(result);
        }
        storage::TransactionOutcome::Rollback(Ok(()))
    });
    execution_results
}

pub fn convert_call_info_to_function_invocation<T>(call_info: &CallInfo) -> Result<FunctionInvocation, Error<T>> {
    let mut inner_calls = vec![];
    for call in &call_info.inner_calls {
        inner_calls.push(convert_call_info_to_function_invocation::<T>(&call)?);
    }

    Ok(FunctionInvocation {
        contract_address: call_info.call.storage_address.0.0.into(),
        entry_point_selector: call_info.call.entry_point_selector.0.into(),
        calldata: call_info.call.calldata.0.iter().map(|x| (*x).into()).collect(),
        caller_address: call_info.call.caller_address.0.0.into(),
        class_hash: call_info.call.class_hash.ok_or(Error::MissingClassHashInCallInfo)?.0.into(),
        entry_point_type: call_info.call.entry_point_type,
        call_type: call_info.call.call_type,
        result: call_info.execution.retdata.0.iter().map(|x| (*x).into()).collect(),
        calls: inner_calls,
        events: call_info.execution.events.iter().map(|event| event.event.clone()).collect(),
        // TODO: implement messages for simulate
        messages: vec![],
    })
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
    Ok(ExecuteInvocation::Success(convert_call_info_to_function_invocation::<T>(call_info)?))
}
