use mp_starknet::crypto::merkle_patricia_tree::merkle_tree::ProofNode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use starknet_core::types::{BlockId, Event, FeeEstimate, FieldElement, MsgToL1};
use starknet_api::deprecated_contract_class::EntryPointType;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RpcGetProofInput {
    /// Block to prove
    pub block_id: BlockId,
    /// Address of the contract to prove the storage of
    pub contract_address: FieldElement,
    /// Storage keys to be proven
    /// More info can be found [here](https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/)
    /// storage_var address is the sn_keccak of the name hashed with the pedersen hash of the keys
    ///
    /// e.g balance_of(key1: felt, key2: felt) -> pedersen("balance_of", pedersen("key1",
    /// pedersen("key2")))
    pub keys: Vec<FieldElement>,
}

/// Holds the membership/non-membership of a contract and its associated contract contract if the
/// contract exists.
#[derive(Debug, Serialize)]
#[skip_serializing_none]
pub struct RpcGetProofOutput {
    /// The global state commitment for Starknet 0.11.0 blocks onwards, if absent the hash
    /// of the first node in the [contract_proof](RpcGetProofOutput#contract_proof) is the global
    /// state commitment.
    pub state_commitment: Option<FieldElement>,
    /// Required to verify that the hash of the class commitment and the root of the
    /// [contract_proof](RpcGetProofOutput::contract_proof) matches the
    /// [state_commitment](Self#state_commitment). Present only for Starknet blocks 0.11.0 onwards.
    pub class_commitment: Option<FieldElement>,

    /// Membership / Non-membership proof for the queried contract
    pub contract_proof: Vec<ProofNode>,

    /// Additional contract data if it exists.
    pub contract_data: Option<ContractData>,
}

/// Holds the data and proofs for a specific contract.
#[derive(Debug, Serialize)]
pub struct ContractData {
    /// Required to verify the contract state hash to contract root calculation.
    pub class_hash: FieldElement,
    /// Required to verify the contract state hash to contract root calculation.
    pub nonce: FieldElement,

    /// Root of the Contract state tree
    pub root: FieldElement,

    /// This is currently just a constant = 0, however it might change in the future.
    pub contract_state_hash_version: FieldElement,

    /// The proofs associated with the queried storage values
    pub storage_proofs: Vec<Vec<ProofNode>>,
}

/// TODO: remove the following types once they are implemented in starknet-rs/starknet-api

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub enum CallType {
    LibraryCall,
    Call,
}

#[derive(Debug, Serialize)]
pub struct FunctionInvocation {
    pub caller_address: FieldElement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_hash: Option<FieldElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point_type: Option<EntryPointType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_type: Option<CallType>,
    pub result: Vec<FieldElement>,

    pub contract_address: FieldElement,
    pub entry_point_selector: FieldElement,
    pub calldata: Vec<FieldElement>,

    pub calls: Vec<FunctionInvocation>,
    pub events: Vec<Event>,
    pub messages: Vec<MsgToL1>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionError {
    pub revert_reason: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ExecuteInvocation {
    FunctionInvocation(FunctionInvocation),
    ExecutionError(ExecutionError)
}

#[derive(Debug, Serialize)]
pub struct InvokeTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_invocation: Option<ExecuteInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Serialize)]
pub struct DeployAccountTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constructor_invocation: Option<FunctionInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Serialize)]
pub struct L1HandlerTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Serialize)]
pub struct DeclareTransactionTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_invocation: Option<FunctionInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_transfer_invocation: Option<FunctionInvocation>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TransactionTrace {
    InvokeTransactionTrace(InvokeTransactionTrace),
    DeployAccountTransactionTrace(DeployAccountTransactionTrace),
    L1HandlerTransactionTrace(L1HandlerTransactionTrace),
    DeclareTransactionTrace(DeclareTransactionTrace),
}

/// The execution trace and consumed resources of the required transactions
#[derive(Debug, Serialize)]
pub struct SimulateTransactionResult {
    pub transaction_trace: TransactionTrace,
    pub fee_estimation: FeeEstimate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SimulateTransactionFlag {
    SkipValidate,
    SkipExecute,  // removed in 0.4.0
    SkipFeeCharge,  // added in 0.4.0
}