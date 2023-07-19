use mp_starknet::transaction::types::{TransactionExecutionInfoWrapper};
use starknet_core::types::{FeeEstimate, FieldElement, MsgToL1, Event};
use starknet_api::hash::StarkFelt;
use starknet_api::api_core::{ContractAddress, ClassHash, EntryPointSelector};
use starknet_api::transaction::{Calldata, EventContent, EthAddress};
use blockifier::execution::entry_point::{CallInfo, CallType as BlockifierCallType, Retdata, MessageToL1};

use crate::types::{SimulateTransactionResult, TransactionTrace, InvokeTransactionTrace, FunctionInvocation, ExecuteInvocation, ExecutionError, CallType};

pub fn get_fee_estimate(execution_info: &TransactionExecutionInfoWrapper) -> FeeEstimate {
    FeeEstimate {
        overall_fee: execution_info.actual_fee.0 as u64,
        gas_price: 0,
        // TODO: actual_resources are not passed atm because it's tricky to implement SCALE on hashmap
        // Alternatively we can aggregate consumed gas from all the call_info structures
        gas_consumed: 0,
    }
}

/// Converts StarkFelt used in Blockifier/StarknetAPI to FieldElement from starknet-rs
pub fn convert_felt(felt: StarkFelt) -> FieldElement {
    FieldElement::from_byte_slice_be(felt.bytes())
        .expect("Expected just to be diff representations of [u8; 32]")
}

/// Converts ContractAddress used in Blockifier/StarknetAPI to FieldElement from starknet-rs
pub fn convert_contract_address(contract_address: ContractAddress) -> FieldElement {
    convert_felt(*contract_address.0.key())
}

/// Converts ClassHash used in Blockifier/StarknetAPI to FieldElement from starknet-rs
pub fn convert_class_hash(class_hash: ClassHash) -> FieldElement {
    convert_felt(class_hash.0)
}

/// Converts CallType used in Blockifier/StarknetAPI to a local type compliant with the JSON-RPC spec
pub fn convert_call_type(call_type: BlockifierCallType) -> CallType {
    match call_type {
        BlockifierCallType::Call => CallType::Call,
        BlockifierCallType::Delegate => CallType::LibraryCall
    }
}

/// Converts Retdata used in Blockifier/StarknetAPI to an array of field elements
pub fn convert_execution_result(retdata: Retdata) -> Vec<FieldElement> {
    retdata.0
        .into_iter()
        .map(convert_felt)
        .collect::<Vec<FieldElement>>()
}

/// Converts EntryPointSelector used in Blockifier/StarknetAPI to FieldElement from starknet-rs
pub fn convert_entry_point_selector(entry_point_selector: EntryPointSelector) -> FieldElement {
    convert_felt(entry_point_selector.0)
}

pub fn convert_calldata(calldata: Calldata) -> Vec<FieldElement> {
    calldata.0
        .iter()
        .map(|felt| convert_felt(felt.clone()))
        .collect::<Vec<FieldElement>>()
}

pub fn convert_event(content: EventContent, from_address: FieldElement) -> Event {
    Event {
        from_address,
        keys: content.keys.into_iter().map(|k| convert_felt(k.0)).collect::<Vec<_>>(),
        data: content.data.0.into_iter().map(|v| convert_felt(v)).collect::<Vec<_>>(),
    }
}

pub fn convert_eth_address(address: EthAddress) -> FieldElement {
    FieldElement::from_byte_slice_be(address.0.as_bytes())
        .expect("Failed to cast Eth address to field element")
    
}

pub fn convert_message(message: MessageToL1, from_address: FieldElement) -> MsgToL1 {
    MsgToL1 {
        from_address,
        to_address: convert_eth_address(message.to_address),
        payload: message.payload.0.into_iter().map(convert_felt).collect::<Vec<_>>(),
    }
}

impl From<CallInfo> for FunctionInvocation {
    fn from(call_info: CallInfo) -> Self {
        // NOTE: there's also optional code address which is None for library calls
        let contract_address = convert_contract_address(call_info.call.storage_address);
        Self {
            caller_address: convert_contract_address(call_info.call.caller_address),
            class_hash: call_info.call.class_hash.map(convert_class_hash),
            entry_point_type: Some(call_info.call.entry_point_type),
            call_type: Some(convert_call_type(call_info.call.call_type)),
            result: convert_execution_result(call_info.execution.retdata),
            contract_address: contract_address.clone(),
            entry_point_selector: convert_entry_point_selector(call_info.call.entry_point_selector),
            calldata: convert_calldata(call_info.call.calldata),
            calls: call_info.inner_calls
                .into_iter()
                .map(|call| call.into())
                .collect::<Vec<_>>(),
            events: call_info.execution.events
                .into_iter()
                .map(|e| convert_event(e.event, contract_address.clone()))
                .collect::<Vec<_>>(),
            messages: call_info.execution.l2_to_l1_messages
                .into_iter()
                .map(|m| convert_message(m.message, contract_address.clone()))
                .collect::<Vec<_>>(),
        }
    }
}

impl From<CallInfo> for ExecuteInvocation {
    fn from(call_info: CallInfo) -> Self {
        if call_info.execution.failed {
            Self::ExecutionError(ExecutionError {
                revert_reason: format!("TODO: revert reason")
            })
        } else {
            Self::FunctionInvocation(call_info.into())
        }
    }
}

impl From<TransactionExecutionInfoWrapper> for InvokeTransactionTrace {
    fn from(execution_info: TransactionExecutionInfoWrapper) -> Self {
        InvokeTransactionTrace {
            validate_invocation: execution_info.validate_call_info.map(|info| info.into()),
            execute_invocation: execution_info.execute_call_info.map(|info| info.into()),
            fee_transfer_invocation: execution_info.fee_transfer_call_info.map(|info| info.into()),
        }
    }
}

impl From<TransactionExecutionInfoWrapper> for TransactionTrace {
    fn from(execution_info: TransactionExecutionInfoWrapper) -> Self {
        match &execution_info.execute_call_info {
            Some(_) => {
                // TODO: handle DeployTransactionCall (constructor call)
                Self::InvokeTransactionTrace(execution_info.into())
            },
            None => {
                unimplemented!("DeclareTransactionTrace")
            }
        }
    }
}

impl From<TransactionExecutionInfoWrapper> for SimulateTransactionResult {
    fn from(execution_info: TransactionExecutionInfoWrapper) -> Self {
        let fee_estimate = get_fee_estimate(&execution_info);
        SimulateTransactionResult {
            transaction_trace: execution_info.into(),
            fee_estimation: fee_estimate
        }
    }
}
