use mp_starknet::crypto::merkle_patricia_tree::merkle_tree::ProofNode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use starknet_core::types::{BlockId, FieldElement};

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
