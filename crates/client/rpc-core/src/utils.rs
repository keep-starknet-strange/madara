use std::collections::HashMap;

use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use cairo_vm::types::program::Program;
use frame_support::inherent::BlockT;
use mp_digest_log::find_starknet_block;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::{
    ContractClassWrapper, EntryPointTypeWrapper, EntryPointWrapper, EntrypointMapWrapper, Felt252Wrapper,
};
use mp_starknet::transaction::types::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction};
use sp_api::HeaderT;
use sp_blockchain::HeaderBackend;
use sp_runtime::BoundedVec;
use starknet_core::types::{
    BroadcastedTransaction, CompressedLegacyContractClass, ContractClass, FromByteArrayError, LegacyContractEntryPoint,
    LegacyEntryPointsByType,
};

/// Returns a `ContractClass` from a `ContractClassWrapper`
pub fn to_rpc_contract_class(contract_class_wrapped: ContractClassWrapper) -> Result<ContractClass> {
    let entry_points_by_type = to_legacy_entry_points_by_type(&contract_class_wrapped.entry_points_by_type)?;

    let program: Program = contract_class_wrapped.program.into();
    let compressed_program = compress_and_encode_base64(&program.to_bytes())?;

    Ok(ContractClass::Legacy(CompressedLegacyContractClass {
        program: compressed_program.as_bytes().to_vec(),
        entry_points_by_type,
        abi: None, // TODO: add ABI
    }))
}

/// Returns a base64 encoded and compressed string of the input bytes
pub(crate) fn compress_and_encode_base64(data: &[u8]) -> Result<String> {
    let data_compressed = compress(data)?;
    Ok(encode_base64(&data_compressed))
}

/// Returns a compressed vector of bytes
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut gzip_encoder, data)?;
    Ok(gzip_encoder.finish()?)
}

/// Returns a base64 encoded string of the input bytes
pub(crate) fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
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
pub fn to_tx(request: BroadcastedTransaction, chain_id: &str) -> Result<Transaction> {
    match request {
        BroadcastedTransaction::Invoke(invoke_tx) => {
            InvokeTransaction::try_from(invoke_tx).map(|inner| inner.from_invoke(chain_id))
        }
        BroadcastedTransaction::Declare(declare_tx) => {
            DeclareTransaction::try_from(declare_tx).map(|inner| inner.from_declare(chain_id))
        }
        BroadcastedTransaction::DeployAccount(deploy_account_tx) => {
            DeployAccountTransaction::try_from(deploy_account_tx)
                .and_then(|inner| inner.from_deploy(chain_id).map_err(|e| anyhow!(e)))
        }
    }
}

/// Returns a [Result<LegacyEntryPointsByType>] (blockifier type) from a [EntrypointMapWrapper]
/// (internal type)
fn to_legacy_entry_points_by_type(entries: &EntrypointMapWrapper) -> Result<LegacyEntryPointsByType> {
    let constructor = entries.0
        .get(&EntryPointTypeWrapper::Constructor).ok_or(anyhow!("Missing constructor entry point"))? // TODO: change to StarknetError
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    let external = entries
        .0
        .get(&EntryPointTypeWrapper::External)
        .ok_or(anyhow!("Missing external entry point"))?
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    let l1_handler = entries
        .0
        .get(&EntryPointTypeWrapper::L1Handler)
        .ok_or(anyhow!("Missing l1 handler entry point"))?
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    Ok(LegacyEntryPointsByType { constructor, external, l1_handler })
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
