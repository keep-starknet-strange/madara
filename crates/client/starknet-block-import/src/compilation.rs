use blockifier::execution::contract_class::{ContractClass as BlockifierCasmClass, EntryPointV1};
use cairo_lang_starknet::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint as SierraEntryPoint,
    ContractEntryPoints as SierraEntryPoints,
};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use mc_rpc::casm_contract_class_to_compiled_class;
use mp_felt::Felt252Wrapper;
use num_bigint::BigUint;
use starknet_api::deprecated_contract_class::EntryPointType as DeprecatedEntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::state::{
    ContractClass as BlockifierSierraClass, EntryPoint as BlockifierEntryPoint,
    EntryPointType as BlockifierEntryPointType,
};
use starknet_core::types::contract::{
    CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList, ComputeClassHashError,
};
use starknet_core::types::{FieldElement, FromByteArrayError};

#[derive(Debug, thiserror::Error)]
pub enum CompilationError {
    #[error(transparent)]
    ComputeClassHash(#[from] ComputeClassHashError),
    #[error(transparent)]
    SierraCompilation(#[from] StarknetSierraCompilationError),
    #[error("Contract class V0 is not supported")]
    UnsupportedClassV0,
    #[error("Unexpected relocatable while converting program to bytecode")]
    UnexpectedRelocatable,
    #[error("Failed to parse felt from bytes: {0}")]
    FeltFromBytes(#[from] FromByteArrayError),
}

pub(crate) fn blockifier_casm_class_to_compiled_class_hash(
    casm_class: BlockifierCasmClass,
) -> Result<FieldElement, CompilationError> {
    match casm_class {
        BlockifierCasmClass::V0(_) => Err(CompilationError::UnsupportedClassV0),
        BlockifierCasmClass::V1(class) => {
            let mut entry_points_by_type = class.entry_points_by_type.clone();
            let compiled_class = CompiledClass {
                bytecode: cairo_vm_program_to_bytecode(&class.program)?,
                entry_points_by_type: CompiledClassEntrypointList {
                    external: entry_points_by_type
                        .remove(&DeprecatedEntryPointType::External)
                        .map_or(vec![], convert_casm_entry_points),
                    l1_handler: entry_points_by_type
                        .remove(&DeprecatedEntryPointType::L1Handler)
                        .map_or(vec![], convert_casm_entry_points),
                    constructor: entry_points_by_type
                        .remove(&DeprecatedEntryPointType::Constructor)
                        .map_or(vec![], convert_casm_entry_points),
                },
                // The rest of the fields do not contribute to the class hash
                prime: Default::default(),
                compiler_version: Default::default(),
                hints: Default::default(),
                pythonic_hints: Default::default(),
            };
            compiled_class.class_hash().map_err(Into::into)
        }
    }
}

pub(crate) fn blockifier_sierra_class_to_compiled_class_hash(
    sierra_class: BlockifierSierraClass,
) -> Result<FieldElement, CompilationError> {
    let BlockifierSierraClass { sierra_program, mut entry_point_by_type, .. } = sierra_class;

    let sierra_contract_class = SierraContractClass {
        sierra_program: sierra_program.iter().map(stark_felt_to_biguint_as_hex).collect(),
        entry_points_by_type: SierraEntryPoints {
            external: entry_point_by_type
                .remove(&BlockifierEntryPointType::External)
                .map_or(vec![], convert_sierra_entry_points),
            l1_handler: entry_point_by_type
                .remove(&BlockifierEntryPointType::L1Handler)
                .map_or(vec![], convert_sierra_entry_points),
            constructor: entry_point_by_type
                .remove(&BlockifierEntryPointType::Constructor)
                .map_or(vec![], convert_sierra_entry_points),
        },
        // The rest of the fields are not used for compilation
        sierra_program_debug_info: None,
        contract_class_version: Default::default(),
        abi: None,
    };

    let casm_contract_class = sierra_contract_class.into_casm_contract_class(false)?;
    let compiled_class = casm_contract_class_to_compiled_class(&casm_contract_class);
    compiled_class.class_hash().map_err(Into::into)
}

pub fn convert_sierra_entry_points(entry_points: Vec<BlockifierEntryPoint>) -> Vec<SierraEntryPoint> {
    entry_points
        .into_iter()
        .map(|entry_point| SierraEntryPoint {
            selector: stark_felt_to_biguint(&entry_point.selector.0),
            function_idx: entry_point.function_idx.0 as usize,
        })
        .collect()
}

pub fn convert_casm_entry_points(entry_points: Vec<EntryPointV1>) -> Vec<CompiledClassEntrypoint> {
    entry_points
        .into_iter()
        .map(|entry_point| CompiledClassEntrypoint {
            builtins: entry_point.builtins.into_iter().map(normalize_builtin_name).collect(),
            offset: entry_point.offset.0 as u64,
            selector: entry_point.selector.0.into(),
        })
        .collect()
}

pub(crate) fn cairo_vm_program_to_bytecode(program: &Program) -> Result<Vec<FieldElement>, CompilationError> {
    let mut bytecode = Vec::with_capacity(program.data_len());
    for item in program.iter_data() {
        match item {
            MaybeRelocatable::Int(felt) => bytecode.push(Felt252Wrapper::from(felt.clone()).into()),
            MaybeRelocatable::RelocatableValue(_) => return Err(CompilationError::UnexpectedRelocatable),
        }
    }
    Ok(bytecode)
}

pub fn stark_felt_to_biguint(felt: &StarkFelt) -> BigUint {
    BigUint::from_bytes_be(felt.bytes())
}

pub fn stark_felt_to_biguint_as_hex(felt: &StarkFelt) -> BigUintAsHex {
    BigUintAsHex { value: stark_felt_to_biguint(felt) }
}

// CairoVM adds "_builtin" suffix to builtin names.
// Need to remove it because it affects class hash.
fn normalize_builtin_name(builtin: String) -> String {
    builtin.strip_suffix("_builtin").map(Into::into).unwrap_or(builtin)
}

#[cfg(test)]
mod test {
    use blockifier::execution::contract_class::{ContractClass as BlockifierCasmClass, ContractClassV1};
    use mc_rpc::flattened_sierra_to_sierra_contract_class;
    use starknet_core::types::contract::SierraClass;
    use starknet_core::types::FieldElement;

    use super::{blockifier_casm_class_to_compiled_class_hash, blockifier_sierra_class_to_compiled_class_hash};

    #[test]
    fn test_blockifier_casm_class_to_compiled_class_hash() {
        // starkli class-hash
        // crates/client/starknet-block-import/tests/counter0/counter0.compiled_contract_class.json
        let expected_class_hash =
            FieldElement::from_hex_be("0x065f93ec23a940ec285a12359778a0865dd20deceec9be7c6e000257e7b0e009").unwrap();
        let casm_class = BlockifierCasmClass::V1(
            ContractClassV1::try_from_json_string(include_str!(
                "../tests/counter0/counter0.compiled_contract_class.json"
            ))
            .unwrap(),
        );
        let casm_class_hash = blockifier_casm_class_to_compiled_class_hash(casm_class).unwrap();
        assert_eq!(expected_class_hash, casm_class_hash);
    }

    #[test]
    fn test_blockifier_sierra_class_to_compiled_class_hash() {
        let expected_class_hash =
            FieldElement::from_hex_be("0x065f93ec23a940ec285a12359778a0865dd20deceec9be7c6e000257e7b0e009").unwrap();
        let sierra_class: SierraClass =
            serde_json::from_str(include_str!("../tests/counter0/counter0.contract_class.json")).unwrap();
        let blockifier_sierra_class = flattened_sierra_to_sierra_contract_class(sierra_class.flatten().unwrap().into());
        let casm_class_hash = blockifier_sierra_class_to_compiled_class_hash(blockifier_sierra_class).unwrap();
        assert_eq!(expected_class_hash, casm_class_hash);
    }
}
