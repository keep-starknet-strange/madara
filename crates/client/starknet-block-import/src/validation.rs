use std::sync::Arc;

use blockifier::transaction::transactions::DeclareTransaction;
use mc_db::sierra_classes_db::SierraClassesDb;
use sp_consensus::Error as ConsensusError;

use crate::compilation::{
    blockifier_casm_class_to_compiled_class_hash, blockifier_sierra_class_to_compiled_class_hash,
    stark_felt_to_field_element,
};

pub fn validate_declare_transaction(
    declare: DeclareTransaction,
    sierra_classes_db: Arc<SierraClassesDb>,
) -> Result<(), ConsensusError> {
    // 0. Check if it's declare V2 or higher and extract Casm class hash
    let expected_casm_class_hash = match declare.tx() {
        starknet_api::transaction::DeclareTransaction::V2(v2) => stark_felt_to_field_element(&v2.compiled_class_hash.0),
        starknet_api::transaction::DeclareTransaction::V3(v3) => stark_felt_to_field_element(&v3.compiled_class_hash.0),
        _ => return Ok(()),
    };

    // 1. Check if we have the according Sierra class stored locally (should have been saved during the
    //    add declare rpc call)
    let sierra_class = sierra_classes_db
        .get_sierra_class(declare.class_hash())
        .map_err(|e| ConsensusError::Other(Box::new(e)))?
        .ok_or_else(|| {
            ConsensusError::StateUnavailable(format!("Could not find Sierra class locally {:?}", declare.class_hash()))
        })?;

    // 2. Check if Casm class matches the compiled class hash in the transaction
    let compiled_casm_class_hash = blockifier_casm_class_to_compiled_class_hash(declare.class_info.contract_class())
        .map_err(|e| ConsensusError::Other(Box::new(e)))?;
    if compiled_casm_class_hash != expected_casm_class_hash {
        return Err(ConsensusError::ClientImport(format!(
            "Mismatched class hash (compiling casm class from extrinsic): expected (in tx) {0:x}, got {1:x}",
            expected_casm_class_hash, compiled_casm_class_hash
        )));
    }

    // 3. Compile Sierra class to Casm class and check class hashes again
    let compiled_casm_class_hash_from_sierra_class =
        blockifier_sierra_class_to_compiled_class_hash(sierra_class).map_err(|e| ConsensusError::Other(Box::new(e)))?;
    if compiled_casm_class_hash_from_sierra_class != expected_casm_class_hash {
        return Err(ConsensusError::ClientImport(format!(
            "Mismatched class hash (compiling sierra class from local db): expected (in tx) {0:x}, got {1:x}",
            expected_casm_class_hash, compiled_casm_class_hash_from_sierra_class
        )));
    }

    Ok(())
}
