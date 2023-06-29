use std::collections::HashMap;

use anyhow::{anyhow, Result};
use blockifier::execution::contract_class::ContractClass as BlockifierContractClass;
use mp_digest_log::find_starknet_block;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{
    BroadcastedTransactionConversionErrorWrapper, DeclareTransaction, DeployAccountTransaction, InvokeTransaction,
    Transaction,
};
use sp_api::{BlockT, HeaderT};
use sp_blockchain::HeaderBackend;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_core::types::{
    BroadcastedTransaction, CompressedLegacyContractClass, ContractClass, FieldElement, FromByteArrayError,
    LegacyContractEntryPoint, LegacyEntryPointsByType,
};

/// Returns a [`ContractClass`] from a [`BlockifierContractClass`]
pub fn to_rpc_contract_class(contract_class: BlockifierContractClass) -> Result<ContractClass> {
    match contract_class {
        BlockifierContractClass::V0(contract_class) => {
            let entry_points_by_type = to_legacy_entry_points_by_type(&contract_class.entry_points_by_type)?;
            let compressed_program = compress(&contract_class.program.to_bytes())?;
            Ok(ContractClass::Legacy(CompressedLegacyContractClass {
                program: compressed_program,
                entry_points_by_type,
                // FIXME 723
                abi: None,
            }))
        }
        BlockifierContractClass::V1(_) => todo!("V1 contract class"),
    }
}

/// Returns a compressed vector of bytes
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut gzip_encoder, data)?;
    Ok(gzip_encoder.finish()?)
}

/// Converts a broadcasted transaction to a transaction
/// Supports `Invoke`, `Declare` and `DeployAccount` transactions
///
/// # Arguments
///
/// * `request` - The broadcasted transaction to convert
///
/// # Returns
///
/// * `Transaction` - The converted transaction
pub fn to_tx(
    request: BroadcastedTransaction,
    chain_id: Felt252Wrapper,
) -> Result<Transaction, BroadcastedTransactionConversionErrorWrapper> {
    match request {
        BroadcastedTransaction::Invoke(invoke_tx) => {
            InvokeTransaction::try_from(invoke_tx).map(|inner| inner.from_invoke(chain_id))
        }
        BroadcastedTransaction::Declare(declare_tx) => {
            DeclareTransaction::try_from(declare_tx).map(|inner| inner.from_declare(chain_id))
        }
        BroadcastedTransaction::DeployAccount(deploy_account_tx) => {
            DeployAccountTransaction::try_from(deploy_account_tx).and_then(|inner| {
                inner
                    .from_deploy(chain_id)
                    .map_err(BroadcastedTransactionConversionErrorWrapper::TransactionConversionError)
            })
        }
    }
}

/// Returns a [Result<LegacyEntryPointsByType>] (starknet-rs type) from a [HashMap<EntryPointType,
/// Vec<EntryPoint>>]
fn to_legacy_entry_points_by_type(
    entries: &HashMap<EntryPointType, Vec<EntryPoint>>,
) -> Result<LegacyEntryPointsByType> {
    fn collect_entry_points(
        entries: &HashMap<EntryPointType, Vec<EntryPoint>>,
        entry_point_type: EntryPointType,
    ) -> Result<Vec<LegacyContractEntryPoint>> {
        Ok(entries
            .get(&entry_point_type)
            .ok_or(anyhow!("Missing {:?} entry point", entry_point_type))?
            .iter()
            .map(|e| to_legacy_entry_point(e.clone()))
            .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?)
    }

    let constructor = collect_entry_points(entries, EntryPointType::Constructor)?;
    let external = collect_entry_points(entries, EntryPointType::External)?;
    let l1_handler = collect_entry_points(entries, EntryPointType::L1Handler)?;

    Ok(LegacyEntryPointsByType { constructor, external, l1_handler })
}

/// Returns a [LegacyContractEntryPoint] (starknet-rs) from a [EntryPoint] (starknet-api)
fn to_legacy_entry_point(entry_point: EntryPoint) -> Result<LegacyContractEntryPoint, FromByteArrayError> {
    let selector = FieldElement::from_bytes_be(&entry_point.selector.0.0)?;
    let offset = entry_point.offset.0 as u64;
    Ok(LegacyContractEntryPoint { selector, offset })
}

/// Returns the current Starknet block from the block header's digest
pub fn get_block_by_block_hash<B, C>(client: &C, block_hash: <B as BlockT>::Hash) -> Option<StarknetBlock>
where
    B: BlockT,
    C: HeaderBackend<B>,
{
    let header = client.header(block_hash).ok().flatten()?;
    let digest = header.digest();
    let block = find_starknet_block(digest).ok()?;
    Some(block)
}
