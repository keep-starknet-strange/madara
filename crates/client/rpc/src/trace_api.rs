use blockifier::execution::contract_class::{ContractClass, ContractClassV1};
use blockifier::execution::entry_point::CallInfo;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_genesis_data_provider::GenesisProvider;
use mc_rpc_core::utils::{blockifier_to_rpc_state_diff_types, get_block_by_block_hash};
use mc_rpc_core::{StarknetReadRpcApiServer, StarknetTraceRpcApiServer};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::{SimulationFlags, TransactionSimulationResult};
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::{DeclareTransaction, Transaction, TxType, UserOrL1HandlerTransaction, UserTransaction};
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::{Backend, BlockBackend, StorageProvider};
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::api_core::ClassHash;
use starknet_core::types::{
    BlockId, BroadcastedTransaction, DeclareTransactionTrace, DeployAccountTransactionTrace, ExecuteInvocation,
    FeeEstimate, InvokeTransactionTrace, L1HandlerTransactionTrace, RevertedInvocation, SimulatedTransaction,
    SimulationFlag, StateDiff, TransactionTrace, TransactionTraceWithHash,
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
        let best_block_hash = self.get_best_block_hash();

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

        let simulated_transactions =
            tx_execution_infos_to_simulated_transactions(tx_types, res).map_err(StarknetRpcApiError::from)?;

        Ok(simulated_transactions)
    }

    async fn trace_block_transactions(&self, block_id: BlockId) -> RpcResult<Vec<TransactionTraceWithHash>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("Block not found: '{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash).map_err(|e| {
            error!("Failed to get block for block hash {substrate_block_hash}: '{e}'");
            StarknetRpcApiError::InternalServerError
        })?;

        let block_transactions = starknet_block
            .transactions()
            .iter()
            .map(|tx| match tx {
                Transaction::Invoke(invoke_tx) => {
                    RpcResult::Ok(UserOrL1HandlerTransaction::User(UserTransaction::Invoke(invoke_tx.clone())))
                }
                Transaction::DeployAccount(deploy_account_tx) => {
                    Ok(UserOrL1HandlerTransaction::User(UserTransaction::DeployAccount(deploy_account_tx.clone())))
                }
                Transaction::Declare(declare_tx, _) => {
                    let class_hash = ClassHash::from(*declare_tx.class_hash());

                    match declare_tx {
                        DeclareTransaction::V0(_) | DeclareTransaction::V1(_) => {
                            let contract_class = self
                                .overrides
                                .for_block_hash(self.client.as_ref(), substrate_block_hash)
                                .contract_class_by_class_hash(substrate_block_hash, class_hash)
                                .ok_or_else(|| {
                                    error!("Failed to retrieve contract class from hash '{class_hash}'");
                                    StarknetRpcApiError::InternalServerError
                                })?;

                            Ok(UserOrL1HandlerTransaction::User(UserTransaction::Declare(
                                declare_tx.clone(),
                                contract_class,
                            )))
                        }
                        DeclareTransaction::V2(tx) => {
                            let contract_class = self
                                .backend
                                .sierra_classes()
                                .get_sierra_class(class_hash)
                                .map_err(|e| {
                                    error!("Failed to fetch sierra class with hash {class_hash}: {e}");
                                    StarknetRpcApiError::InternalServerError
                                })?
                                .ok_or_else(|| {
                                    error!("The sierra class with hash {class_hash} is not present in db backend");
                                    StarknetRpcApiError::InternalServerError
                                })?;
                            let contract_class = mp_transactions::utils::sierra_to_casm_contract_class(contract_class)
                                .map_err(|e| {
                                    error!("Failed to convert the SierraContractClass to CasmContractClass: {e}");
                                    StarknetRpcApiError::InternalServerError
                                })?;
                            let contract_class =
                                ContractClass::V1(ContractClassV1::try_from(contract_class).map_err(|e| {
                                    error!(
                                        "Failed to convert the compiler CasmContractClass to blockifier \
                                         CasmContractClass: {e}"
                                    );
                                    StarknetRpcApiError::InternalServerError
                                })?);

                            Ok(UserOrL1HandlerTransaction::User(UserTransaction::Declare(
                                declare_tx.clone(),
                                contract_class,
                            )))
                        }
                    }
                }
                Transaction::L1Handler(handle_l1_message_tx) => {
                    let chain_id = self.chain_id()?.0.into();
                    let tx_hash = handle_l1_message_tx.compute_hash::<H>(chain_id, false);
                    let paid_fee =
                        self.backend.l1_handler_paid_fee().get_fee_paid_for_l1_handler_tx(tx_hash.into()).map_err(
                            |e| {
                                error!("Failed to retrieve fee paid on l1 for tx with hash `{tx_hash:?}`: {e}");
                                StarknetRpcApiError::InternalServerError
                            },
                        )?;

                    Ok(UserOrL1HandlerTransaction::L1Handler(handle_l1_message_tx.clone(), paid_fee))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let previous_block_substrate_hash = {
            let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash).map_err(|e| {
                error!("Failed to starknet block for substate block with hash {substrate_block_hash}: {e}");
                StarknetRpcApiError::InternalServerError
            })?;
            let block_number = starknet_block.header().block_number;
            let previous_block_number = block_number - 1;
            self.substrate_block_hash_from_starknet_block(BlockId::Number(previous_block_number)).map_err(|e| {
                error!("Failed to retrieve previous block substrate hash: {e}");
                StarknetRpcApiError::InternalServerError
            })
        }?;

        let execution_infos = self
            .client
            .runtime_api()
            .re_execute_transactions(previous_block_substrate_hash, block_transactions.clone())
            .map_err(|e| {
                error!("Failed to execute runtime API call: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|e| {
                error!("Failed to reexecute the block transactions: {e:?}");
                StarknetRpcApiError::InternalServerError
            })?
            .map_err(|_| {
                error!(
                    "One of the transaction failed during it's reexecution. This should not happen, as the block has \
                     already been executed successfully in the past. There is a bug somewhere."
                );
                StarknetRpcApiError::InternalServerError
            })?;

        let storage_override = self.overrides.for_block_hash(self.client.as_ref(), substrate_block_hash);
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let traces = execution_infos
            .into_iter()
            .enumerate()
            .map(|(tx_idx, (tx_exec_info, commitment_state_diff))| {
                let state_diff = blockifier_to_rpc_state_diff_types(commitment_state_diff)
                    .map_err(|_| ConvertCallInfoToExecuteInvocationError::ConvertStateDiffFailed)?;
                tx_execution_infos_to_tx_trace(
                    // Safe to unwrap coz re_execute returns exactly one ExecutionInfo for each tx
                    TxType::from(block_transactions.get(tx_idx).unwrap()),
                    &tx_exec_info,
                    Some(state_diff),
                )
                .map(|trace_root| TransactionTraceWithHash {
                    transaction_hash: block_transactions[tx_idx].compute_hash::<H>(chain_id, false).into(),
                    trace_root,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(StarknetRpcApiError::from)?;

        Ok(traces)
    }
}

#[derive(Error, Debug)]
pub enum ConvertCallInfoToExecuteInvocationError {
    #[error("One of the simulated transaction failed")]
    TransactionExecutionFailed,
    #[error(transparent)]
    GetFunctionInvocation(#[from] TryFuntionInvocationFromCallInfoError),
    #[error("Failed to convert state diff")]
    ConvertStateDiffFailed,
}

impl From<ConvertCallInfoToExecuteInvocationError> for StarknetRpcApiError {
    fn from(err: ConvertCallInfoToExecuteInvocationError) -> Self {
        match err {
            ConvertCallInfoToExecuteInvocationError::TransactionExecutionFailed => StarknetRpcApiError::ContractError,
            ConvertCallInfoToExecuteInvocationError::GetFunctionInvocation(_) => {
                StarknetRpcApiError::InternalServerError
            }
            ConvertCallInfoToExecuteInvocationError::ConvertStateDiffFailed => StarknetRpcApiError::InternalServerError,
        }
    }
}

fn collect_call_info_ordered_messages(call_info: &CallInfo) -> Vec<starknet_core::types::OrderedMessage> {
    call_info
        .execution
        .l2_to_l1_messages
        .iter()
        .map(|message| starknet_core::types::OrderedMessage {
            order: message.order as u64,
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

fn try_get_function_invocation_from_call_info(
    call_info: &CallInfo,
) -> Result<starknet_core::types::FunctionInvocation, TryFuntionInvocationFromCallInfoError> {
    let messages = collect_call_info_ordered_messages(call_info);
    let events = blockifier_to_starknet_rs_ordered_events(&call_info.execution.events);

    let inner_calls =
        call_info.inner_calls.iter().map(try_get_function_invocation_from_call_info).collect::<Result<_, _>>()?;

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

    // The class hash in the call_info is computed during execution and will be set here.
    let class_hash = call_info.call.class_hash.expect("Class hash should be computed after execution").0.into();

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

fn tx_execution_infos_to_tx_trace(
    tx_type: TxType,
    tx_exec_info: &TransactionExecutionInfo,
    state_diff: Option<StateDiff>,
) -> Result<TransactionTrace, ConvertCallInfoToExecuteInvocationError> {
    // If simulated with `SimulationFlag::SkipValidate` this will be `None`
    // therefore we cannot unwrap it
    let validate_invocation =
        tx_exec_info.validate_call_info.as_ref().map(try_get_function_invocation_from_call_info).transpose()?;
    // If simulated with `SimulationFlag::SkipFeeCharge` this will be `None`
    // therefore we cannot unwrap it
    let fee_transfer_invocation =
        tx_exec_info.fee_transfer_call_info.as_ref().map(try_get_function_invocation_from_call_info).transpose()?;

    let tx_trace = match tx_type {
        TxType::Invoke => TransactionTrace::Invoke(InvokeTransactionTrace {
            validate_invocation,
            execute_invocation: if let Some(e) = &tx_exec_info.revert_error {
                ExecuteInvocation::Reverted(RevertedInvocation { revert_reason: e.clone() })
            } else {
                ExecuteInvocation::Success(try_get_function_invocation_from_call_info(
                    // Safe to unwrap because is only `None`  for `Declare` txs
                    tx_exec_info.execute_call_info.as_ref().unwrap(),
                )?)
            },
            fee_transfer_invocation,
            state_diff,
        }),
        TxType::Declare => TransactionTrace::Declare(DeclareTransactionTrace {
            validate_invocation,
            fee_transfer_invocation,
            state_diff,
        }),
        TxType::DeployAccount => {
            TransactionTrace::DeployAccount(DeployAccountTransactionTrace {
                validate_invocation,
                constructor_invocation: try_get_function_invocation_from_call_info(
                    // Safe to unwrap because is only `None` for `Declare` txs
                    tx_exec_info.execute_call_info.as_ref().unwrap(),
                )?,
                fee_transfer_invocation,
                state_diff,
            })
        }
        TxType::L1Handler => TransactionTrace::L1Handler(L1HandlerTransactionTrace {
            function_invocation: try_get_function_invocation_from_call_info(
                // Safe to unwrap because is only `None` for `Declare` txs
                tx_exec_info.execute_call_info.as_ref().unwrap(),
            )?,
            state_diff: None,
        }),
    };

    Ok(tx_trace)
}

fn tx_execution_infos_to_simulated_transactions(
    tx_types: Vec<TxType>,
    transaction_execution_results: Vec<(CommitmentStateDiff, TransactionSimulationResult)>,
) -> Result<Vec<SimulatedTransaction>, ConvertCallInfoToExecuteInvocationError> {
    let mut results = vec![];
    for (tx_type, (state_diff, res)) in tx_types.into_iter().zip(transaction_execution_results.into_iter()) {
        match res {
            Ok(tx_exec_info) => {
                let state_diff = blockifier_to_rpc_state_diff_types(state_diff)
                    .map_err(|_| ConvertCallInfoToExecuteInvocationError::ConvertStateDiffFailed)?;

                let transaction_trace = tx_execution_infos_to_tx_trace(tx_type, &tx_exec_info, Some(state_diff))?;
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
