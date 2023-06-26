use anyhow::{anyhow, Result};
use cairo_vm::types::program::Program;
use mp_digest_log::find_starknet_block;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::{
    ContractClassWrapper, EntryPointTypeWrapper, EntrypointMapWrapper, Felt252Wrapper,
};
use mp_starknet::transaction::types::{
    BroadcastedTransactionConversionErrorWrapper, DeclareTransaction, DeployAccountTransaction, InvokeTransaction,
    Transaction,
};
use sp_api::{BlockT, HeaderT};
use sp_blockchain::HeaderBackend;
use starknet_core::types::{
    BroadcastedTransaction, CompressedLegacyContractClass, ContractClass, FromByteArrayError, LegacyContractEntryPoint,
    LegacyEntryPointsByType,
};

/// Returns a [`ContractClass`] from a [`ContractClassWrapper`]
pub fn to_rpc_contract_class(contract_class_wrapped: ContractClassWrapper) -> Result<ContractClass> {
    let entry_points_by_type = to_legacy_entry_points_by_type(&contract_class_wrapped.entry_points_by_type)?;

    let program: Program =
        contract_class_wrapped.program.try_into().map_err(|_| anyhow!("Contract Class conversion failed."))?;
    let compressed_program = compress(&program.to_bytes())?;

    Ok(ContractClass::Legacy(CompressedLegacyContractClass {
        program: compressed_program,
        entry_points_by_type,
        // FIXME 723
        abi: None,
    }))
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

/// Returns a [Result<LegacyEntryPointsByType>] (blockifier type) from a [EntrypointMapWrapper]
/// (internal type)
fn to_legacy_entry_points_by_type(entries: &EntrypointMapWrapper) -> Result<LegacyEntryPointsByType> {
    fn collect_entry_points(
        entries: &EntrypointMapWrapper,
        entry_point_type: EntryPointTypeWrapper,
    ) -> Result<Vec<LegacyContractEntryPoint>> {
        Ok(entries
            .0
            .get(&entry_point_type)
            .ok_or(anyhow!("Missing {:?} entry point", entry_point_type))?
            .iter()
            .map(|e| (e.clone()).try_into())
            .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?)
    }

    let constructor = collect_entry_points(entries, EntryPointTypeWrapper::Constructor)?;
    let external = collect_entry_points(entries, EntryPointTypeWrapper::External)?;
    let l1_handler = collect_entry_points(entries, EntryPointTypeWrapper::L1Handler)?;

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
