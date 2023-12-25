#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use blockifier::execution::entry_point::{CallInfo, OrderedL2ToL1Message};
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionResult;
use mp_felt::{Felt252Wrapper, UfeHex};
use mp_state::StateDiff;
use starknet_api::api_core::EthAddress;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::EventContent;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimulationFlags {
    pub skip_validate: bool,
    pub skip_fee_charge: bool,
}

impl From<Vec<SimulationFlag>> for SimulationFlags {
    fn from(flags: Vec<SimulationFlag>) -> Self {
        let mut skip_validate = false;
        let mut skip_fee_charge = false;

        for flag in flags {
            match flag {
                SimulationFlag::SkipValidate => skip_validate = true,
                SimulationFlag::SkipFeeCharge => skip_fee_charge = true,
            }
            if skip_validate && skip_fee_charge {
                break;
            }
        }

        Self { skip_validate, skip_fee_charge }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SimulationFlag {
    #[serde(rename = "SKIP_VALIDATE")]
    SkipValidate,
    #[serde(rename = "SKIP_FEE_CHARGE")]
    SkipFeeCharge,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SimulatedTransaction {
    /// The transaction's trace
    pub transaction_trace: TransactionTrace,
    /// The transaction's resources and fee
    pub fee_estimation: FeeEstimate,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[serde(untagged)]
pub enum TransactionTrace {
    Invoke(InvokeTransactionTrace),
    DeployAccount(DeployAccountTransactionTrace),
    L1Handler(L1HandlerTransactionTrace),
    Declare(DeclareTransactionTrace),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FeeEstimate {
    /// The Ethereum gas cost of the transaction (see
    /// https://docs.starknet.io/docs/fees/fee-mechanism for more info)
    pub gas_consumed: u64,
    /// The gas price (in gwei) that was used in the cost estimation
    pub gas_price: u64,
    /// The estimated fee for the transaction (in gwei), product of gas_consumed and gas_price
    pub overall_fee: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeclareTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
    /// The state diffs induced by the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<StateDiff>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvokeTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    pub execute_invocation: ExecuteInvocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
    /// The state diffs induced by the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<StateDiff>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeployAccountTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    /// The trace of the __execute__ call or constructor call, depending on the transaction type
    /// (none for declare transactions)
    pub constructor_invocation: FunctionInvocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
    /// The state diffs induced by the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<StateDiff>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct L1HandlerTransactionTrace {
    /// The trace of the __execute__ call or constructor call, depending on the transaction type
    /// (none for declare transactions)
    pub function_invocation: FunctionInvocation,
    /// The state diffs induced by the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<StateDiff>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MessageToL1 {
    /// The address of the L2 contract sending the message
    #[serde_as(as = "UfeHex")]
    pub from_address: Felt252Wrapper,
    /// The target L1 address the message is sent to
    pub to_address: EthAddress,
    /// The payload of the message
    #[serde_as(as = "Vec<UfeHex>")]
    pub payload: Vec<Felt252Wrapper>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionInvocation {
    /// Contract address
    #[serde_as(as = "UfeHex")]
    pub contract_address: Felt252Wrapper,
    /// Entry point selector
    #[serde_as(as = "UfeHex")]
    pub entry_point_selector: Felt252Wrapper,
    /// The parameters passed to the function
    #[serde_as(as = "Vec<UfeHex>")]
    pub calldata: Vec<Felt252Wrapper>,
    /// The address of the invoking contract. 0 for the root invocation
    #[serde_as(as = "UfeHex")]
    pub caller_address: Felt252Wrapper,
    /// The hash of the class being called
    #[serde_as(as = "Option<UfeHex>")]
    pub class_hash: Option<Felt252Wrapper>,
    pub entry_point_type: EntryPointType,
    pub call_type: CallType,
    /// The value returned from the function invocation
    #[serde_as(as = "Vec<UfeHex>")]
    pub result: Vec<Felt252Wrapper>,
    /// The calls made by this invocation
    pub calls: Vec<FunctionInvocation>,
    /// The events emitted in this invocation
    pub events: Vec<EventContent>,
    /// The messages sent by this invocation to L1
    pub messages: Vec<MessageToL1>,
}

impl TryFrom<&CallInfo> for FunctionInvocation {
    type Error = TransactionExecutionError;

    fn try_from(call_info: &CallInfo) -> TransactionExecutionResult<FunctionInvocation> {
        let messages = ordered_l2_to_l1_messages(call_info);

        let inner_calls = call_info
            .inner_calls
            .iter()
            .map(|call| call.try_into())
            .collect::<Result<_, TransactionExecutionError>>()?;

        call_info.get_sorted_l2_to_l1_payloads_length()?;

        Ok(FunctionInvocation {
            contract_address: call_info.call.storage_address.0.0.into(),
            entry_point_selector: call_info.call.entry_point_selector.0.into(),
            calldata: call_info.call.calldata.0.iter().map(|x| (*x).into()).collect(),
            caller_address: call_info.call.caller_address.0.0.into(),
            class_hash: call_info.call.class_hash.map(|x| x.0.into()),
            entry_point_type: call_info.call.entry_point_type,
            call_type: call_info.call.call_type.into(),
            result: call_info.execution.retdata.0.iter().map(|x| (*x).into()).collect(),
            calls: inner_calls,
            events: call_info.execution.events.iter().map(|event| event.event.clone()).collect(),
            messages,
        })
    }
}

fn ordered_l2_to_l1_messages(call_info: &CallInfo) -> Vec<MessageToL1> {
    let mut messages = BTreeMap::new();

    for call in call_info.into_iter() {
        for OrderedL2ToL1Message { order, message } in &call.execution.l2_to_l1_messages {
            messages.insert(
                order,
                MessageToL1 {
                    payload: message.payload.0.iter().map(|x| (*x).into()).collect(),
                    to_address: message.to_address,
                    from_address: call.call.storage_address.0.0.into(),
                },
            );
        }
    }

    messages.into_values().collect()
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[serde(untagged)]
pub enum ExecuteInvocation {
    Success(FunctionInvocation),
    Reverted(RevertedInvocation),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RevertedInvocation {
    /// The revert reason for the failed execution
    pub revert_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallType {
    #[serde(rename = "CALL")]
    Call,
    #[serde(rename = "LIBRARY_CALL")]
    LibraryCall,
}

impl From<blockifier::execution::entry_point::CallType> for CallType {
    fn from(value: blockifier::execution::entry_point::CallType) -> Self {
        use blockifier::execution::entry_point::CallType::*;
        match value {
            Call => Self::Call,
            Delegate => Self::LibraryCall,
        }
    }
}
