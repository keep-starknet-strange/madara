use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::state::cached_state::{CachedState, CommitmentStateDiff};
use blockifier::state::state_api::State;
use blockifier::transaction::objects::TransactionExecutionInfo;
use frame_support::storage;
use mp_felt::Felt252Wrapper;
use mp_simulations::PlaceHolderErrorTypeForFailedStarknetExecution;
use mp_transactions::execution::{Execute, ExecutionConfig};
use mp_transactions::UserTransaction;
use sp_runtime::DispatchError;

use crate::blockifier_state_adapter::{BlockifierStateAdapter, CachedBlockifierStateAdapter};
use crate::{pallet, Error};

pub fn execute_txs_and_rollback<T: pallet::Config>(
    txs: &Vec<UserTransaction>,
    block_context: &BlockContext,
    chain_id: Felt252Wrapper,
    execution_config: &mut ExecutionConfig,
) -> Result<
    Vec<(Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>, CommitmentStateDiff)>,
    Error<T>,
> {
    let mut execution_results = vec![];

    storage::transactional::with_transaction(|| {
        for tx in txs {
            execution_config.set_offset_version(tx.offset_version());
            let mut cached_state =
                CachedBlockifierStateAdapter(CachedState::from(BlockifierStateAdapter::<T>::default()));
            let result = match tx {
                UserTransaction::Declare(tx, contract_class) => tx
                    .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                    .and_then(|exec| {
                        let execution_result = exec.execute(&mut cached_state, block_context, execution_config);
                        execution_result
                    }),
                UserTransaction::DeployAccount(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    let mut cached_state =
                        CachedBlockifierStateAdapter(CachedState::from(BlockifierStateAdapter::<T>::default()));
                    let execution_result = executable.execute(&mut cached_state, block_context, execution_config);
                    execution_result
                }
                UserTransaction::Invoke(tx) => {
                    let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                    let mut cached_state =
                        CachedBlockifierStateAdapter(CachedState::from(BlockifierStateAdapter::<T>::default()));
                    let execution_result = executable.execute(&mut cached_state, block_context, execution_config);
                    execution_result
                }
            }
            .map_err(|_| PlaceHolderErrorTypeForFailedStarknetExecution);

            execution_results.push((result, cached_state.to_state_diff()));
        }
        storage::TransactionOutcome::Rollback(Result::<_, DispatchError>::Ok(()))
    })
    .map_err(|_| Error::<T>::FailedToCreateATransactionalStorageExecution)?;

    Ok(execution_results)
}
