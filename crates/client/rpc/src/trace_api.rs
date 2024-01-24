use blockifier::execution::entry_point::CallInfo;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_genesis_data_provider::GenesisProvider;
use mc_rpc_core::utils::blockifier_to_rpc_state_diff_types;
use mc_rpc_core::{StarknetReadRpcApiServer, StarknetTraceRpcApiServer};
use mc_storage::StorageOverride;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags};
use mp_transactions::TxType;
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::{Backend, BlockBackend, StorageProvider};
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::{
    BlockId, BroadcastedTransaction, DeclareTransactionTrace, DeployAccountTransactionTrace, ExecuteInvocation,
    FeeEstimate, InvokeTransactionTrace, RevertedInvocation, SimulatedTransaction, SimulationFlag, TransactionTrace,
};
use starknet_ff::FieldElement;
use thiserror::Error;

use crate::errors::StarknetRpcApiError;
use crate::Starknet;

#[async_trait]
#[allow(unused_variables)]
impl<A, B, BE, G, C, P, H> StarknetTraceRpcApiServer for Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B> + 'static,
    G: GenesisProvider + Send + Sync + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    H: HasherT + Send + Sync + 'static,
{
    async fn simulate_transactions(
        &self,
        block_id: BlockId,
        transactions: Vec<BroadcastedTransaction>,
        simulation_flags: Vec<SimulationFlag>,
    ) -> RpcResult<Vec<SimulatedTransaction>> {
        let substrate_block_hash =
            self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| StarknetRpcApiError::BlockNotFound)?;
        let chain_id = Felt252Wrapper(self.chain_id()?.0);
        let best_block_hash = self.client.info().best_hash;

        let tx_type_and_tx_iterator = transactions.into_iter().map(|tx| match tx {
            BroadcastedTransaction::Invoke(invoke_tx) => invoke_tx.try_into().map(|tx| (TxType::Invoke, tx)),
            BroadcastedTransaction::Declare(declare_tx) => declare_tx.try_into().map(|tx| (TxType::Declare, tx)),
            BroadcastedTransaction::DeployAccount(deploy_account_tx) => {
                deploy_account_tx.try_into().map(|tx| (TxType::DeployAccount, tx))
            }
        });
        let (tx_types, user_transactions) =
            itertools::process_results(tx_type_and_tx_iterator, |iter| iter.unzip::<_, _, Vec<_>, Vec<_>>()).map_err(
                |e| {
                    error!("Failed to convert BroadcastedTransaction to UserTransaction: {e}");
                    StarknetRpcApiError::InternalServerError
                },
            )?;

        let simulation_flags = SimulationFlags::from(simulation_flags);

        let res = self
            .client
            .runtime_api()
            .simulate_transactions(substrate_block_hash, user_transactions, simulation_flags)
            .map_err(|e| {
                error!("Request parameters error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to call function: {:#?}", e);
                StarknetRpcApiError::ContractError
            })?;

        let storage_override = self.overrides.for_block_hash(self.client.as_ref(), substrate_block_hash);
        let simulated_transactions =
            tx_execution_infos_to_simulated_transactions(&**storage_override, substrate_block_hash, tx_types, res)
                .map_err(|e| match e {
                    ConvertCallInfoToExecuteInvocationError::TransactionExecutionFailed => {
                        StarknetRpcApiError::ContractError
                    }
                    ConvertCallInfoToExecuteInvocationError::GetFunctionInvocation(_) => {
                        StarknetRpcApiError::InternalServerError
                    }
                })?;

        Ok(simulated_transactions)
    }
}

#[derive(Error, Debug)]
pub enum ConvertCallInfoToExecuteInvocationError {
    #[error("One of the simulated transaction failed")]
    TransactionExecutionFailed,
    #[error(transparent)]
    GetFunctionInvocation(#[from] TryFuntionInvocationFromCallInfoError),
}

fn collect_call_info_ordered_messages(call_info: &CallInfo) -> Vec<starknet_core::types::OrderedMessage> {
    call_info
        .execution
        .l2_to_l1_messages
        .iter()
        .enumerate()
        .map(|(index, message)| starknet_core::types::OrderedMessage {
            order: index as u64,
            payload: message.message.payload.0.iter().map(|x| (*x).into()).collect(),
            to_address: FieldElement::from_byte_slice_be(message.message.to_address.0.to_fixed_bytes().as_slice())
                .unwrap(),
            from_address: call_info.call.storage_address.0.0.into(),
        })
        .collect()
}

fn blockifier_to_starknet_rs_ordered_events(
    ordered_events: &[blockifier::execution::entry_point::OrderedEvent],
) -> Vec<starknet_core::types::OrderedEvent> {
    ordered_events
        .iter()
        .map(|event| starknet_core::types::OrderedEvent {
            order: event.order as u64, // Convert usize to u64
            keys: event.event.keys.iter().map(|key| FieldElement::from_byte_slice_be(key.0.bytes()).unwrap()).collect(),
            data: event
                .event
                .data
                .0
                .iter()
                .map(|data_item| FieldElement::from_byte_slice_be(data_item.bytes()).unwrap())
                .collect(),
        })
        .collect()
}

#[derive(Error, Debug)]
pub enum TryFuntionInvocationFromCallInfoError {
    #[error(transparent)]
    TransactionExecution(#[from] TransactionExecutionError),
    #[error("No contract found at the Call contract_address")]
    ContractNotFound,
}

fn try_get_funtion_invocation_from_call_info<B: BlockT>(
    storage_override: &dyn StorageOverride<B>,
    substrate_block_hash: B::Hash,
    call_info: &CallInfo,
) -> Result<starknet_core::types::FunctionInvocation, TryFuntionInvocationFromCallInfoError> {
    let messages = collect_call_info_ordered_messages(call_info);
    let events = blockifier_to_starknet_rs_ordered_events(&call_info.execution.events);

    let inner_calls = call_info
        .inner_calls
        .iter()
        .map(|call| try_get_funtion_invocation_from_call_info(storage_override, substrate_block_hash, call))
        .collect::<Result<_, _>>()?;

    call_info.get_sorted_l2_to_l1_payloads_length()?;

    let entry_point_type = match call_info.call.entry_point_type {
        starknet_api::deprecated_contract_class::EntryPointType::Constructor => {
            starknet_core::types::EntryPointType::Constructor
        }
        starknet_api::deprecated_contract_class::EntryPointType::External => {
            starknet_core::types::EntryPointType::External
        }
        starknet_api::deprecated_contract_class::EntryPointType::L1Handler => {
            starknet_core::types::EntryPointType::L1Handler
        }
    };

    let call_type = match call_info.call.call_type {
        blockifier::execution::entry_point::CallType::Call => starknet_core::types::CallType::Call,
        blockifier::execution::entry_point::CallType::Delegate => starknet_core::types::CallType::Delegate,
    };

    // Blockifier call info does not give use the class_hash "if it can be deducted from the storage
    // address". We have to do this decution ourselves here
    let class_hash = if let Some(class_hash) = call_info.call.class_hash {
        class_hash.0.into()
    } else {
        let class_hash = storage_override
            .contract_class_hash_by_address(substrate_block_hash, call_info.call.storage_address)
            .ok_or_else(|| TryFuntionInvocationFromCallInfoError::ContractNotFound)?;

        FieldElement::from_byte_slice_be(class_hash.0.bytes()).unwrap()
    };

    Ok(starknet_core::types::FunctionInvocation {
        contract_address: call_info.call.storage_address.0.0.into(),
        entry_point_selector: call_info.call.entry_point_selector.0.into(),
        calldata: call_info.call.calldata.0.iter().map(|x| (*x).into()).collect(),
        caller_address: call_info.call.caller_address.0.0.into(),
        class_hash,
        entry_point_type,
        call_type,
        result: call_info.execution.retdata.0.iter().map(|x| (*x).into()).collect(),
        calls: inner_calls,
        events,
        messages,
    })
}

fn tx_execution_infos_to_simulated_transactions<B: BlockT>(
    storage_override: &dyn StorageOverride<B>,
    substrate_block_hash: B::Hash,
    tx_types: Vec<TxType>,
    transaction_execution_results: Vec<(
        Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>,
        CommitmentStateDiff,
    )>,
) -> Result<Vec<SimulatedTransaction>, ConvertCallInfoToExecuteInvocationError> {
    let mut results = vec![];
    for (tx_type, (res, state_diff)) in tx_types.iter().zip(transaction_execution_results.iter()) {
        match res {
            Ok(tx_exec_info) => {
                // If simulated with `SimulationFlag::SkipValidate` this will be `None`
                // therefore we cannot unwrap it
                let validate_invocation = tx_exec_info
                    .validate_call_info
                    .as_ref()
                    .map(|call_info| {
                        try_get_funtion_invocation_from_call_info(storage_override, substrate_block_hash, call_info)
                    })
                    .transpose()?;
                // If simulated with `SimulationFlag::SkipFeeCharge` this will be `None`
                // therefore we cannot unwrap it
                let fee_transfer_invocation = tx_exec_info
                    .fee_transfer_call_info
                    .as_ref()
                    .map(|call_info| {
                        try_get_funtion_invocation_from_call_info(storage_override, substrate_block_hash, call_info)
                    })
                    .transpose()?;

                let transaction_trace = match tx_type {
                    TxType::Invoke => TransactionTrace::Invoke(InvokeTransactionTrace {
                        validate_invocation,
                        execute_invocation: if let Some(e) = &tx_exec_info.revert_error {
                            ExecuteInvocation::Reverted(RevertedInvocation { revert_reason: e.clone() })
                        } else {
                            ExecuteInvocation::Success(try_get_funtion_invocation_from_call_info(
                                storage_override,
                                substrate_block_hash,
                                // Safe to unwrap because is only `None`  for `Declare` txs
                                tx_exec_info.execute_call_info.as_ref().unwrap(),
                            )?)
                        },
                        fee_transfer_invocation,
                        state_diff: Some(state_diff),
                    }),
                    TxType::Declare => TransactionTrace::Declare(DeclareTransactionTrace {
                        validate_invocation,
                        fee_transfer_invocation,
                        state_diff: Some(state_diff),
                    }),
                    TxType::DeployAccount => {
                        TransactionTrace::DeployAccount(DeployAccountTransactionTrace {
                            validate_invocation,
                            constructor_invocation: try_get_funtion_invocation_from_call_info(
                                storage_override,
                                substrate_block_hash,
                                // Safe to unwrap because is only `None`  for `Declare` txs
                                tx_exec_info.execute_call_info.as_ref().unwrap(),
                            )?,
                            fee_transfer_invocation,
                            state_diff: Some(state_diff),
                        })
                    }
                    TxType::L1Handler => unreachable!("L1Handler transactions cannot be simulated"),
                };

                let gas_consumed =
                    tx_exec_info.execute_call_info.as_ref().map(|x| x.execution.gas_consumed).unwrap_or_default();
                let overall_fee = tx_exec_info.actual_fee.0 as u64;
                // TODO: Shouldn't the gas price be taken from the block header instead?
                let gas_price = if gas_consumed > 0 { overall_fee / gas_consumed } else { 0 };

                results.push(SimulatedTransaction {
                    transaction_trace,
                    fee_estimation: FeeEstimate { gas_consumed, gas_price, overall_fee },
                });
            }
            Err(_) => {
                return Err(ConvertCallInfoToExecuteInvocationError::TransactionExecutionFailed);
            }
        }
    }

    Ok(results)
}
