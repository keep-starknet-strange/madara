use blockifier::execution::call_info::CallInfo;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::transaction_execution::Transaction;
use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use mc_genesis_data_provider::GenesisProvider;
use mc_rpc_core::utils::{blockifier_to_rpc_state_diff_types, get_block_by_block_hash};
use mc_rpc_core::{StarknetReadRpcApiServer, StarknetTraceRpcApiServer};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_simulations::{SimulationFlags, TransactionSimulationResult};
use mp_transactions::from_broadcasted_transactions::{
    try_declare_tx_from_broadcasted_declare_tx, try_deploy_tx_from_broadcasted_deploy_tx,
    try_invoke_tx_from_broadcasted_invoke_tx,
};
use mp_transactions::{get_transaction_hash, TxType};
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::{Backend, BlockBackend, StorageProvider};
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_api::transaction::TransactionHash;
use starknet_core::types::{
    BlockId, BroadcastedTransaction, DeclareTransactionTrace, DeployAccountTransactionTrace, ExecuteInvocation,
    FeeEstimate, InvokeTransactionTrace, L1HandlerTransactionTrace, PriceUnit, RevertedInvocation,
    SimulatedTransaction, SimulationFlag, StateDiff, TransactionTrace, TransactionTraceWithHash,
};
use starknet_ff::FieldElement;
use thiserror::Error;

use crate::errors::StarknetRpcApiError;
use crate::Starknet;

#[async_trait]
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
            self.substrate_block_hash_from_starknet_block(block_id).map_err(|_| StarknetRpcApiError::BlockNotFound)?;
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let mut tx_types = Vec::with_capacity(transactions.len());
        let mut account_transactions = Vec::with_capacity(transactions.len());

        let tx_type_and_tx_iterator = transactions.into_iter().map(|tx| match tx {
            BroadcastedTransaction::Invoke(invoke_tx) => (
                TxType::Invoke,
                try_invoke_tx_from_broadcasted_invoke_tx(invoke_tx, chain_id).map(AccountTransaction::Invoke),
            ),
            BroadcastedTransaction::Declare(declare_tx) => (
                TxType::Declare,
                try_declare_tx_from_broadcasted_declare_tx(declare_tx, chain_id).map(AccountTransaction::Declare),
            ),
            BroadcastedTransaction::DeployAccount(deploy_account_tx) => (
                TxType::DeployAccount,
                try_deploy_tx_from_broadcasted_deploy_tx(deploy_account_tx, chain_id)
                    .map(AccountTransaction::DeployAccount),
            ),
        });

        for (tx_type, account_tx) in tx_type_and_tx_iterator {
            let tx = account_tx.map_err(|e| {
                error!("Failed to convert BroadcastedTransaction to AccountTransaction: {e}");
                StarknetRpcApiError::InternalServerError
            })?;
            account_transactions.push(tx);
            tx_types.push(tx_type);
        }

        let simulation_flags = SimulationFlags::from(simulation_flags);

        let res = self
            .client
            .runtime_api()
            .simulate_transactions(substrate_block_hash, account_transactions, simulation_flags)
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

        let block_transactions = starknet_block.transactions();

        let previous_block_substrate_hash = get_previous_block_substrate_hash(self, substrate_block_hash)?;

        let execution_infos =
            self.re_execute_transactions(previous_block_substrate_hash, vec![], block_transactions.clone())?;

        let traces = Self::execution_info_to_transaction_trace(execution_infos, block_transactions)?;

        Ok(traces)
    }

    async fn trace_transaction(&self, transaction_hash: FieldElement) -> RpcResult<TransactionTrace> {
        let transaction_hash: TransactionHash = Felt252Wrapper::from(transaction_hash).into();

        let substrate_block_hash = self
            .backend
            .mapping()
            .block_hash_from_transaction_hash(transaction_hash)
            .map_err(|e| {
                error!("Failed to get transaction's substrate block hash from mapping_db: {e}");
                StarknetRpcApiError::TxnHashNotFound
            })?
            .ok_or(StarknetRpcApiError::TxnHashNotFound)?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash)?;

        let (txs_before, tx_to_trace) = super::split_block_tx_for_reexecution(&starknet_block, transaction_hash)?;
        let tx_type = TxType::from(&tx_to_trace[0]);

        let previous_block_substrate_hash = get_previous_block_substrate_hash(self, substrate_block_hash)?;

        let (execution_infos, commitment_state_diff) = self
            .re_execute_transactions(previous_block_substrate_hash, txs_before, tx_to_trace)?
            .into_iter()
            .next()
            .unwrap();

        let state_diff = blockifier_to_rpc_state_diff_types(commitment_state_diff.clone())
            .map_err(|_| StarknetRpcApiError::from(ConvertCallInfoToExecuteInvocationError::ConvertStateDiffFailed))?;

        let trace = tx_execution_infos_to_tx_trace(tx_type, &execution_infos, Some(state_diff))
            .map_err(StarknetRpcApiError::from)?;

        Ok(trace)
    }
}

impl<A, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
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
    pub fn re_execute_transactions(
        &self,
        previous_block_substrate_hash: B::Hash,
        transactions_before: Vec<Transaction>,
        transactions_to_trace: Vec<Transaction>,
    ) -> RpcResult<Vec<(TransactionExecutionInfo, CommitmentStateDiff)>> {
        Ok(self
            .client
            .runtime_api()
            .re_execute_transactions(
                previous_block_substrate_hash,
                transactions_before.clone(),
                transactions_to_trace.clone(),
            )
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
            })?)
    }

    fn execution_info_to_transaction_trace(
        execution_infos: Vec<(TransactionExecutionInfo, CommitmentStateDiff)>,
        block_transactions: &[Transaction],
    ) -> RpcResult<Vec<TransactionTraceWithHash>> {
        Ok(execution_infos
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
                    transaction_hash: Felt252Wrapper::from(*get_transaction_hash(&block_transactions[tx_idx])).into(),
                    trace_root,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(StarknetRpcApiError::from)?)
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
            payload: message.message.payload.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect(),
            to_address: FieldElement::from_byte_slice_be(message.message.to_address.0.to_fixed_bytes().as_slice())
                .unwrap(),
            from_address: Felt252Wrapper::from(call_info.call.storage_address).into(),
        })
        .collect()
}

fn blockifier_to_starknet_rs_ordered_events(
    ordered_events: &[blockifier::execution::call_info::OrderedEvent],
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
    let class_hash =
        Felt252Wrapper::from(call_info.call.class_hash.expect("Class hash should be computed after execution")).0;

    Ok(starknet_core::types::FunctionInvocation {
        contract_address: Felt252Wrapper::from(call_info.call.storage_address).into(),
        entry_point_selector: Felt252Wrapper::from(call_info.call.entry_point_selector).into(),
        calldata: call_info.call.calldata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect(),
        caller_address: Felt252Wrapper::from(call_info.call.caller_address).into(),
        class_hash,
        entry_point_type,
        call_type,
        result: call_info.execution.retdata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect(),
        calls: inner_calls,
        events,
        messages,
        execution_resources: vm_to_starknet_rs_exec_resources(&call_info.resources),
    })
}

fn vm_to_starknet_rs_exec_resources(
    resources: &cairo_vm::vm::runners::cairo_runner::ExecutionResources,
) -> starknet_core::types::ExecutionResources {
    starknet_core::types::ExecutionResources {
        steps: resources.n_steps.try_into().unwrap(),
        memory_holes: Some(resources.n_memory_holes.try_into().unwrap()),
        range_check_builtin_applications: resources
            .builtin_instance_counter
            .get("range_check_builtin")
            .map(|&v| v.try_into().unwrap()),
        pedersen_builtin_applications: resources
            .builtin_instance_counter
            .get("pedersen_builtin")
            .map(|&v| v.try_into().unwrap()),
        poseidon_builtin_applications: resources
            .builtin_instance_counter
            .get("poseidon_builtin")
            .map(|&v| v.try_into().unwrap()),
        ec_op_builtin_applications: resources
            .builtin_instance_counter
            .get("ec_op_builtin")
            .map(|&v| v.try_into().unwrap()),
        ecdsa_builtin_applications: resources
            .builtin_instance_counter
            .get("ecdsa_builtin")
            .map(|&v| v.try_into().unwrap()),
        bitwise_builtin_applications: resources
            .builtin_instance_counter
            .get("bitwise_builtin")
            .map(|&v| v.try_into().unwrap()),
        keccak_builtin_applications: resources
            .builtin_instance_counter
            .get("keccak_builtin")
            .map(|&v| v.try_into().unwrap()),
        segment_arena_builtin: resources
            .builtin_instance_counter
            .get("segment_arena_builtin")
            .map(|&v| v.try_into().unwrap()),
    }
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
                    fee_estimation: FeeEstimate {
                        gas_consumed: FieldElement::from(gas_consumed),
                        gas_price: FieldElement::from(gas_price),
                        overall_fee: FieldElement::from(overall_fee),
                        unit: PriceUnit::Wei,
                    },
                });
            }
            Err(_) => {
                return Err(ConvertCallInfoToExecuteInvocationError::TransactionExecutionFailed);
            }
        }
    }

    Ok(results)
}

fn get_previous_block_substrate_hash<A, B, BE, G, C, P, H>(
    starknet: &Starknet<A, B, BE, G, C, P, H>,
    substrate_block_hash: B::Hash,
) -> Result<B::Hash, StarknetRpcApiError>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    H: HasherT + Send + Sync + 'static,
    BE: Backend<B> + 'static,
{
    let starknet_block = get_block_by_block_hash(starknet.client.as_ref(), substrate_block_hash).map_err(|e| {
        error!("Failed to get block for block hash {substrate_block_hash}: '{e}'");
        StarknetRpcApiError::InternalServerError
    })?;
    let block_number = starknet_block.header().block_number;
    let previous_block_number = block_number - 1;
    let substrate_block_hash =
        starknet.substrate_block_hash_from_starknet_block(BlockId::Number(previous_block_number)).map_err(|e| {
            error!("Failed to retrieve previous block substrate hash: {e}");
            StarknetRpcApiError::InternalServerError
        })?;

    Ok(substrate_block_hash)
}
