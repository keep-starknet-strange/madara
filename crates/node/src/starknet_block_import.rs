use std::sync::Arc;

use async_trait::async_trait;
use blockifier::execution::contract_class::{ContractClass as BlockifierCasmClass, EntryPointV1};
use cairo_lang_starknet::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint as SierraEntryPoint,
    ContractEntryPoints as SierraEntryPoints,
};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use madara_runtime::opaque::Block;
use madara_runtime::Hash;
use mc_db::sierra_classes_db::SierraClassesDb;
use mc_rpc::casm_contract_class_to_compiled_class;
use mp_felt::Felt252Wrapper;
use mp_transactions::{DeclareTransaction, DeclareTransactionV2, Transaction};
use num_bigint::{BigInt, BigUint, Sign};
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult, JustificationImport};
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::NumberFor;
use sp_runtime::Justification;
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

use crate::service::FullClient;
use crate::starknet::MadaraBackend;

#[derive(Clone)]
pub struct StarknetBlockImport<I: Clone> {
    inner: I,
    client: Arc<FullClient>,
    madara_backend: Arc<MadaraBackend>,
}

impl<I: BlockImport<Block> + Send + Clone> StarknetBlockImport<I> {
    pub fn new(inner: I, client: Arc<FullClient>, madara_backend: Arc<MadaraBackend>) -> Self {
        Self { inner, client, madara_backend }
    }
}

#[async_trait]
impl<I: BlockImport<Block, Error = ConsensusError> + Send + Clone> BlockImport<Block> for StarknetBlockImport<I> {
    type Error = ConsensusError;

    async fn check_block(&mut self, block: BlockCheckParams<Block>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(&mut self, block: BlockImportParams<Block>) -> Result<ImportResult, Self::Error> {
        log::info!("🐺 Starknet block import: verifying declared CASM classes against Sierra sources");
        if let Some(extrinsics) = &block.body {
            let prev_block_hash = block.header.parent_hash().clone();
            let transactions: Vec<Transaction> = self
                .client
                .runtime_api()
                .extrinsic_filter(prev_block_hash, extrinsics.clone())
                .map_err(|e| ConsensusError::ClientImport(e.to_string()))?;

            for tx in transactions {
                if let Transaction::Declare(DeclareTransaction::V2(declare_v2), casm_class) = tx {
                    verify_declare_v2_transaction(declare_v2, casm_class, self.madara_backend.sierra_classes().clone())?;
                }
            }
        }

        self.inner.import_block(block).await
    }
}

#[async_trait]
impl<I: JustificationImport<Block> + Send + Clone> JustificationImport<Block> for StarknetBlockImport<I> {
    type Error = I::Error;

    async fn on_start(&mut self) -> Vec<(Hash, NumberFor<Block>)> {
        self.inner.on_start().await
    }

    async fn import_justification(
        &mut self,
        hash: Hash,
        number: NumberFor<Block>,
        justification: Justification,
    ) -> Result<(), Self::Error> {
        self.inner.import_justification(hash, number, justification).await
    }
}

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

pub fn verify_declare_v2_transaction(
    declare_v2: DeclareTransactionV2,
    casm_class: BlockifierCasmClass,
    sierra_classes_db: Arc<SierraClassesDb>,
) -> Result<(), ConsensusError> {
    // 1. Check if we have the according Sierra class stored locally (should have been saved during the
    //    add declare rpc call)
    let sierra_class = sierra_classes_db
        .get_sierra_class(declare_v2.class_hash.into())
        .map_err(|e| ConsensusError::Other(Box::new(e)))?
        .ok_or_else(|| {
            ConsensusError::StateUnavailable(format!(
                "Could not find Sierra class locally {:?}",
                declare_v2.class_hash.0
            ))
        })?;
    let expected_casm_class_hash = declare_v2.compiled_class_hash.0;

    // 2. Check if Casm class matches the compiled class hash in the transaction
    let compiled_casm_class_hash = blockifier_casm_class_to_compiled_class_hash(casm_class)
        .map_err(|e| ConsensusError::Other(Box::new(e)))?;
    if compiled_casm_class_hash != expected_casm_class_hash {
        return Err(ConsensusError::ClientImport(format!(
            "Mismatched class hash (compiling casm class from extrinsic): expected (in tx) {0:x}, got {1:x}",
            expected_casm_class_hash, compiled_casm_class_hash
        )));
    }

    // 3. Compile Sierra class to Casm class and check class hashes again
    let compiled_sierra_class_hash = blockifier_sierra_class_to_compiled_class_hash(sierra_class)
        .map_err(|e| ConsensusError::Other(Box::new(e)))?;
    if compiled_sierra_class_hash != expected_casm_class_hash {
        return Err(ConsensusError::ClientImport(format!(
            "Mismatched class hash (compiling sierra class from local db): expected (in tx) {0:x}, got {1:x}",
            expected_casm_class_hash, compiled_sierra_class_hash
        )));
    }

    Ok(())
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
            builtins: entry_point.builtins,
            offset: entry_point.offset.0 as u64,
            selector: entry_point.selector.0.into(),
        })
        .collect()
}

pub(crate) fn cairo_vm_program_to_bytecode(program: &Program) -> Result<Vec<FieldElement>, CompilationError> {
    let mut bytecode = Vec::with_capacity(program.data_len());
    for item in program.data().iter() {
        match item {
            MaybeRelocatable::Int(felt) => bytecode.push(Felt252Wrapper::from(felt.clone()).into()),
            MaybeRelocatable::RelocatableValue(_) => return Err(CompilationError::UnexpectedRelocatable),
        }
    }
    Ok(bytecode)
}

pub fn stark_felt_to_biguint(felt: &StarkFelt) -> BigUint {
    BigInt::from_bytes_be(Sign::Plus, felt.bytes()).to_biguint().unwrap()
}

pub fn stark_felt_to_biguint_as_hex(felt: &StarkFelt) -> BigUintAsHex {
    BigUintAsHex { value: stark_felt_to_biguint(felt) }
}
