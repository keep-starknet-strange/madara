use mp_starknet::crypto::merkle_patricia_tree::merkle_tree::ProofNode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RpcGetProofInput {
    pub block_id: BlockId,
    pub contract_address: FieldElement,
    pub keys: Vec<FieldElement>,
}

/// Holds the membership/non-membership of a contract and its associated contract contract if the
/// contract exists.
#[derive(Debug, Serialize)]
#[skip_serializing_none]
pub struct RpcGetProofOutput {
    /// The global state commitment for Starknet 0.11.0 blocks onwards, if absent the hash
    /// of the first node in the [contract_proof](GetProofOutput#contract_proof) is the global state
    /// commitment.
    state_commitment: Option<FieldElement>,
    /// Required to verify that the hash of the class commitment and the root of the
    /// [contract_proof](GetProofOutput::contract_proof) matches the
    /// [state_commitment](Self#state_commitment). Present only for Starknet blocks 0.11.0 onwards.
    class_commitment: Option<FieldElement>,

    /// Membership / Non-membership proof for the queried contract
    contract_proof: Vec<ProofNode>,

    /// Additional contract data if it exists.
    contract_data: Option<FieldElement>,
}

/// Holds the data and proofs for a specific contract.
#[derive(Debug, Serialize)]
pub struct ContractData {
    /// Required to verify the contract state hash to contract root calculation.
    class_hash: FieldElement,
    /// Required to verify the contract state hash to contract root calculation.
    nonce: FieldElement,

    /// Root of the Contract state tree
    root: FieldElement,

    /// This is currently just a constant = 0, however it might change in the future.
    contract_state_hash_version: FieldElement,

    /// The proofs associated with the queried storage values
    storage_proofs: Vec<Vec<ProofNode>>,
}
