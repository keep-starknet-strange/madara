use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use frame_support::inherent::BlockT;
use mp_digest_log::find_starknet_block;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::{
    ContractClassWrapper, EntryPointTypeWrapper, EntryPointWrapper, Felt252Wrapper, MaxEntryPoints,
};
use mp_starknet::transaction::types::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction};
use sp_api::HeaderT;
use sp_blockchain::HeaderBackend;
use sp_runtime::{BoundedBTreeMap, BoundedVec};
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction,
    BroadcastedTransaction, CompressedLegacyContractClass, ContractClass, FromByteArrayError, LegacyContractEntryPoint,
    LegacyEntryPointsByType, StarknetError,
};

/// Returns a `ContractClass` from a `ContractClassWrapper`
pub fn to_rpc_contract_class(_contract_class_wrapped: ContractClassWrapper) -> Result<ContractClass> {
    todo!()
    // let entry_points_by_type =
    // to_legacy_entry_points_by_type(&_contract_class_wrapped.entry_points_by_type.into())?;

    // let program: Program =
    //     _contract_class_wrapped.program.try_into().map_err(|_| anyhow!("Contract Class conversion
    // failed."))?; let compressed_program = compress_and_encode_base64(&program.to_bytes())?;

    // Ok(ContractClass::Legacy(CompressedLegacyContractClass {
    //     program: compressed_program.as_bytes().to_vec(),
    //     entry_points_by_type,
    //     abi: None, // TODO: add ABI
    // }))
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
pub(crate) fn _encode_base64(data: &[u8]) -> String {
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
        BroadcastedTransaction::Invoke(invoke_tx) => to_invoke_tx(invoke_tx).map(|inner| inner.from_invoke(chain_id)),
        BroadcastedTransaction::Declare(declare_tx) => {
            to_declare_tx(declare_tx).map(|inner| inner.from_declare(chain_id))
        }
        BroadcastedTransaction::DeployAccount(deploy_account_tx) => to_deploy_account_tx(deploy_account_tx)
            .and_then(|inner| inner.from_deploy(chain_id).map_err(|e| anyhow!(e))),
    }
}

/// Converts a broadcasted invoke transaction to an invoke transaction
pub fn to_invoke_tx(tx: BroadcastedInvokeTransaction) -> Result<InvokeTransaction> {
    match tx {
        BroadcastedInvokeTransaction::V0(_) => Err(StarknetError::FailedToReceiveTransaction.into()),
        BroadcastedInvokeTransaction::V1(invoke_tx_v1) => Ok(InvokeTransaction {
            version: 1_u8,
            signature: BoundedVec::try_from(
                invoke_tx_v1.signature.iter().map(|x| (*x).into()).collect::<Vec<Felt252Wrapper>>(),
            )
            .map_err(|e| anyhow!("failed to convert signature: {:?}", e))?,

            sender_address: invoke_tx_v1.sender_address.into(),
            nonce: Felt252Wrapper::from(invoke_tx_v1.nonce),
            calldata: BoundedVec::try_from(
                invoke_tx_v1.calldata.iter().map(|x| (*x).into()).collect::<Vec<Felt252Wrapper>>(),
            )
            .map_err(|e| anyhow!("failed to convert calldata: {:?}", e))?,
            max_fee: Felt252Wrapper::from(invoke_tx_v1.max_fee),
        }),
    }
}

/// Converts a broadcasted deploy account transaction to a deploy account transaction
pub fn to_deploy_account_tx(tx: BroadcastedDeployAccountTransaction) -> Result<DeployAccountTransaction> {
    let contract_address_salt = tx.contract_address_salt.into();

    let account_class_hash = tx.class_hash;

    let signature = tx
        .signature
        .iter()
        .map(|f| (*f).into())
        .collect::<Vec<Felt252Wrapper>>()
        .try_into()
        .map_err(|_| anyhow!("failed to bound signatures Vec<H256> by MaxArraySize"))?;

    let calldata = tx
        .constructor_calldata
        .iter()
        .map(|f| (*f).into())
        .collect::<Vec<Felt252Wrapper>>()
        .try_into()
        .map_err(|_| anyhow!("failed to bound calldata Vec<U256> by MaxCalldataSize"))?;

    let nonce = Felt252Wrapper::from(tx.nonce);
    let max_fee = Felt252Wrapper::from(tx.max_fee);

    Ok(DeployAccountTransaction {
        version: 1_u8,
        calldata,
        salt: contract_address_salt,
        signature,
        account_class_hash: account_class_hash.into(),
        nonce,
        max_fee,
    })
}

/// Converts a broadcasted declare transaction to a declare transaction
pub fn to_declare_tx(_tx: BroadcastedDeclareTransaction) -> Result<DeclareTransaction> {
    todo!();
    // match tx {
    //     BroadcastedDeclareTransaction::V1(declare_tx_v1) => {
    //         let signature = declare_tx_v1
    //             .signature
    //             .iter()
    //             .map(|f| (*f).into())
    //             .collect::<Vec<Felt252Wrapper>>()
    //             .try_into()
    //             .map_err(|_| anyhow!("failed to bound signatures Vec<H256> by MaxArraySize"))?;

    //         // Create a GzipDecoder to decompress the bytes
    //         let mut gz = GzDecoder::new(&declare_tx_v1.contract_class.program[..]);

    //         // Read the decompressed bytes into a Vec<u8>
    //         let mut decompressed_bytes = Vec::new();
    //         std::io::Read::read_to_end(&mut gz, &mut decompressed_bytes)
    //             .map_err(|_| anyhow!("Failed to decompress the contract class program"))?;

    //         // Deserialize it then
    //         let program: Program = Program::from_bytes(&decompressed_bytes, None)
    //             .map_err(|_| anyhow!("Failed to deserialize the contract class program"))?;

    //         Ok(DeclareTransaction {
    //             version: 1_u8,
    //             sender_address: declare_tx_v1.sender_address.into(),
    //             nonce: Felt252Wrapper::from(declare_tx_v1.nonce),
    //             max_fee: Felt252Wrapper::from(declare_tx_v1.max_fee),
    //             signature,
    //             contract_class: ContractClassWrapper {
    //                 program: program.try_into().map_err(|_| anyhow!("Failed to convert program to
    // program wrapper"))?,                 entry_points_by_type:
    // BoundedBTreeMap::try_from(to_btree_map_entrypoints(
    // declare_tx_v1.contract_class.entry_points_by_type.clone(),                 ))
    //                 .unwrap(),
    //             },
    //             compiled_class_hash: Felt252Wrapper::ZERO, // TODO: compute class hash
    //         })
    //     }
    //     BroadcastedDeclareTransaction::V2(_) =>
    // Err(StarknetError::FailedToReceiveTransaction.into()), }
}

/// Returns a btree map of entry point types to entrypoint from deprecated entry point by type
fn to_btree_map_entrypoints(
    entries: LegacyEntryPointsByType,
) -> BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> {
    let mut entry_points_by_type: BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> =
        BTreeMap::new();

    entry_points_by_type.insert(EntryPointTypeWrapper::Constructor, get_entrypoint_value(entries.constructor));
    entry_points_by_type.insert(EntryPointTypeWrapper::External, get_entrypoint_value(entries.external));
    entry_points_by_type.insert(EntryPointTypeWrapper::L1Handler, get_entrypoint_value(entries.l1_handler));
    entry_points_by_type
}

fn to_legacy_entry_points_by_type(
    entries: &BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>>,
) -> Result<LegacyEntryPointsByType> {
    let constructor = entries
        .get(&EntryPointTypeWrapper::Constructor).ok_or(anyhow!("Missing constructor entry point"))? // TODO: change to StarknetError
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    let external = entries
        .get(&EntryPointTypeWrapper::External)
        .ok_or(anyhow!("Missing external entry point"))?
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    let l1_handler = entries
        .get(&EntryPointTypeWrapper::L1Handler)
        .ok_or(anyhow!("Missing l1 handler entry point"))?
        .iter()
        .map(|e| (e.clone()).try_into())
        .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?;

    Ok(LegacyEntryPointsByType { constructor, external, l1_handler })
}

/// Returns a bounded vector of `EntryPointWrapper` from a vector of LegacyContractEntryPoint
fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> BoundedVec<EntryPointWrapper, MaxEntryPoints> {
    // We can unwrap safely as we already checked the length of the vectors
    BoundedVec::try_from(entries.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>()).unwrap()
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
