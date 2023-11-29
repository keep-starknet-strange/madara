use blockifier::execution::entry_point::CallType;
use mp_felt::Felt252Wrapper;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::EventContent;

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
    pub validate_invocation: Option<FunctionInvocation>,
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvokeTransactionTrace {
    pub validate_invocation: Option<FunctionInvocation>,
    pub execute_invocation: ExecuteInvocation,
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeployAccountTransactionTrace {
    pub validate_invocation: Option<FunctionInvocation>,
    /// The trace of the __execute__ call or constructor call, depending on the transaction type
    /// (none for declare transactions)
    pub constructor_invocation: FunctionInvocation,
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct L1HandlerTransactionTrace {
    /// The trace of the __execute__ call or constructor call, depending on the transaction type
    /// (none for declare transactions)
    pub function_invocation: FunctionInvocation,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionInvocation {
    /// Contract address
    pub contract_address: Felt252Wrapper,
    /// Entry point selector
    pub entry_point_selector: Felt252Wrapper,
    /// The parameters passed to the function
    pub calldata: Vec<Felt252Wrapper>,
    /// The address of the invoking contract. 0 for the root invocation
    pub caller_address: Felt252Wrapper,
    /// The hash of the class being called
    pub class_hash: Felt252Wrapper,
    pub entry_point_type: EntryPointType,
    pub call_type: CallType,
    /// The value returned from the function invocation
    pub result: Vec<Felt252Wrapper>,
    /// The calls made by this invocation
    pub calls: Vec<FunctionInvocation>,
    /// The events emitted in this invocation
    pub events: Vec<EventContent>,
    /// The messages sent by this invocation to L1
    pub messages: Vec<MsgToL1>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MsgToL1 {
    /// The address of the L2 contract sending the message
    pub from_address: Felt252Wrapper,
    /// The target L1 address the message is sent to
    pub to_address: Felt252Wrapper,
    /// The payload of the message
    pub payload: Vec<Felt252Wrapper>,
}
