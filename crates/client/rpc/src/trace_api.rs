use blockifier::execution::entry_point::CallInfo;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_genesis_data_provider::GenesisProvider;
use mc_rpc_core::{StarknetReadRpcApiServer, StarknetTraceRpcApiServer};
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

        let simulation_flags: SimulationFlags = simulation_flags.into();

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

        Ok(tx_execution_infos_to_simulated_transactions(tx_types, res).unwrap())
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
    let mut messages = Vec::new();

    for (index, message) in call_info.execution.l2_to_l1_messages.iter().enumerate() {
        messages.push(starknet_core::types::OrderedMessage {
            order: index as u64,
            payload: message.message.payload.0.iter().map(|x| (*x).into()).collect(),
            to_address: FieldElement::from_byte_slice_be(message.message.to_address.0.to_fixed_bytes().as_slice())
                .unwrap(),
            from_address: call_info.call.storage_address.0.0.into(),
        });
    }

    messages
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
}

fn try_get_funtion_invocation_from_call_info(
    call_info: &CallInfo,
) -> Result<starknet_core::types::FunctionInvocation, TryFuntionInvocationFromCallInfoError> {
    let messages = collect_call_info_ordered_messages(call_info);
    let events = blockifier_to_starknet_rs_ordered_events(&call_info.execution.events);

    let inner_calls = call_info
        .inner_calls
        .iter()
        .map(|call| try_get_funtion_invocation_from_call_info(call))
        .collect::<Result<_, TryFuntionInvocationFromCallInfoError>>()?;

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

    Ok(starknet_core::types::FunctionInvocation {
        contract_address: call_info.call.storage_address.0.0.into(),
        entry_point_selector: call_info.call.entry_point_selector.0.into(),
        calldata: call_info.call.calldata.0.iter().map(|x| (*x).into()).collect(),
        caller_address: call_info.call.caller_address.0.0.into(),
        // TODO: Blockifier call info does not give use the class_hash "if it can be deducted from the storage address"
        // We have to do this deductive work ourselves somewhere
        class_hash: call_info.call.class_hash.map(|x| x.0.into()).unwrap_or_default(),
        entry_point_type,
        call_type,
        result: call_info.execution.retdata.0.iter().map(|x| (*x).into()).collect(),
        calls: inner_calls,
        events,
        messages,
    })
}

fn tx_execution_infos_to_simulated_transactions(
    tx_types: Vec<TxType>,
    transaction_execution_results: Vec<
        Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>,
    >,
) -> Result<Vec<SimulatedTransaction>, ConvertCallInfoToExecuteInvocationError> {
    let mut results = vec![];
    for (tx_type, res) in tx_types.iter().zip(transaction_execution_results.iter()) {
        match res {
            Ok(tx_exec_info) => {
                // Both are safe to unwrap because blockifier states that
                // `validate_call_info` and `fee_transfer_call_info` should only be `None` for `L1Handler`
                // transactions
                let validate_invocation =
                    try_get_funtion_invocation_from_call_info(tx_exec_info.validate_call_info.as_ref().unwrap())?;
                let fee_transfer_invocation =
                    try_get_funtion_invocation_from_call_info(tx_exec_info.fee_transfer_call_info.as_ref().unwrap())?;

                let transaction_trace = match tx_type {
                    TxType::Invoke => TransactionTrace::Invoke(InvokeTransactionTrace {
                        validate_invocation: Some(validate_invocation),
                        execute_invocation: if let Some(e) = &tx_exec_info.revert_error {
                            ExecuteInvocation::Reverted(RevertedInvocation { revert_reason: e.clone() })
                        } else {
                            ExecuteInvocation::Success(try_get_funtion_invocation_from_call_info(
                                // Safe to unwrap because is only `None`  for `Declare` txs
                                tx_exec_info.execute_call_info.as_ref().unwrap(),
                            )?)
                        },
                        fee_transfer_invocation: Some(fee_transfer_invocation),
                        // TODO(#1291): Compute state diff correctly
                        state_diff: None,
                    }),
                    TxType::Declare => TransactionTrace::Declare(DeclareTransactionTrace {
                        validate_invocation: Some(validate_invocation),
                        fee_transfer_invocation: Some(fee_transfer_invocation),
                        // TODO(#1291): Compute state diff correctly
                        state_diff: None,
                    }),
                    TxType::DeployAccount => {
                        TransactionTrace::DeployAccount(DeployAccountTransactionTrace {
                            validate_invocation: Some(validate_invocation),
                            constructor_invocation: try_get_funtion_invocation_from_call_info(
                                // Safe to unwrap because is only `None`  for `Declare` txs
                                &tx_exec_info.execute_call_info.as_ref().unwrap(),
                            )?,
                            fee_transfer_invocation: Some(fee_transfer_invocation),
                            // TODO(#1291): Compute state diff correctly
                            state_diff: None,
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
