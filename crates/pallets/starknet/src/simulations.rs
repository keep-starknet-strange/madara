// All the method defined there should only be called from the RuntimeAPI,
// never from some extrinsic body, otherwise they will alter the chain state.
// I can't emphasis enought how important it is.

use alloc::vec::Vec;

use blockifier::transaction::objects::TransactionExecutionInfo;
use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags};
use mp_transactions::execution::Execute;
use mp_transactions::UserTransaction;
use sp_runtime::DispatchError;

use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::execution_config::RuntimeExecutionConfigBuilder;
use crate::{Config, Error, Pallet};

impl<T: Config> Pallet<T> {
    /// Estimate the fee associated with transaction
    pub fn estimate_fee(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, DispatchError> {
        let transactions_len = transactions.len();
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config = RuntimeExecutionConfigBuilder::new::<T>().with_query_mode().build();

        let fee_res_iterator = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());
                let execution_info_res = match tx {
                    UserTransaction::Declare(tx, contract_class) => tx
                        .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                        .and_then(|exec| {
                            exec.execute(&mut BlockifierStateAdapter::<T>::default(), &block_context, &execution_config)
                        }),
                    UserTransaction::DeployAccount(tx) => {
                        let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                        executable.execute(
                            &mut BlockifierStateAdapter::<T>::default(),
                            &block_context,
                            &execution_config,
                        )
                    }
                    UserTransaction::Invoke(tx) => {
                        let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                        executable.execute(
                            &mut BlockifierStateAdapter::<T>::default(),
                            &block_context,
                            &execution_config,
                        )
                    }
                };

                match execution_info_res {
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
    ) -> Result<Vec<Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>>, DispatchError>
    {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut execution_config =
            RuntimeExecutionConfigBuilder::new::<T>().with_simulation_mode(simulation_flags).build();

        let tx_execution_results = transactions
            .into_iter()
            .map(|tx| {
                execution_config.set_offset_version(tx.offset_version());
                match tx {
                    UserTransaction::Declare(tx, contract_class) => tx
                        .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), tx.offset_version())
                        .and_then(|exec| {
                            exec.execute(&mut BlockifierStateAdapter::<T>::default(), &block_context, &execution_config)
                        }),
                    UserTransaction::DeployAccount(tx) => {
                        let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                        executable.execute(
                            &mut BlockifierStateAdapter::<T>::default(),
                            &block_context,
                            &execution_config,
                        )
                    }
                    UserTransaction::Invoke(tx) => {
                        let executable = tx.into_executable::<T::SystemHash>(chain_id, tx.offset_version());
                        executable.execute(
                            &mut BlockifierStateAdapter::<T>::default(),
                            &block_context,
                            &execution_config,
                        )
                    }
                }
                .map_err(|e| {
                    log::error!("Transaction execution failed during simulation: {e}");
                    PlaceHolderErrorTypeForFailedStarknetExecution
                })
            })
            .collect();

        Ok(tx_execution_results)
    }
}
