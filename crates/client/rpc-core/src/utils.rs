use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use base64::engine::general_purpose;
use base64::Engine;
use frame_support::storage::bounded_vec::BoundedVec;
use mp_starknet::execution::types::{ContractClassWrapper, EntryPointTypeWrapper, EntryPointWrapper, MaxEntryPoints};
use mp_starknet::transaction::types::InvokeTransaction;
use sp_core::{H256, U256};
use starknet::providers::jsonrpc::models::{BroadcastedInvokeTransaction, ErrorCode};

use super::types::{DeprecatedEntryPointsByType, RPCContractClass};

/// Returns a `ContractClassWrapper` from a `RPCContractClass`
pub fn to_rpc_contract_class(_contract_class_wrapped: ContractClassWrapper) -> Result<RPCContractClass> {
    Ok(RPCContractClass::ContractClass(Default::default()))
}

/// Returns a base64 encoded and compressed string of the input bytes
pub(crate) fn _compress_and_encode_base64(data: &[u8]) -> Result<String> {
    let data_compressed = _compress(data)?;
    Ok(_encode_base64(&data_compressed))
}

/// Returns a compressed vector of bytes
pub(crate) fn _compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut gzip_encoder, data)?;
    Ok(gzip_encoder.finish()?)
}

/// Returns a base64 encoded string of the input bytes
pub(crate) fn _encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Add 0x prefix to hex string
pub(crate) fn add_prefix(s: &str) -> String {
    format!("0x{}", s)
}

/// Remove leading zeros from str
pub(crate) fn remove_leading_zeros(s: &str) -> &str {
    s.trim_start_matches('0')
}

/// Returns a deprecated entry point by type from hash map of entry point types to entrypoint
pub(crate) fn _to_deprecated_entrypoint_by_type(
    entries: BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>>,
) -> DeprecatedEntryPointsByType {
    let mut constructor = vec![];
    let mut external = vec![];
    let mut l_1_handler = vec![];
    entries.into_iter().for_each(|(entry_type, entry_points)| match entry_type {
        EntryPointTypeWrapper::Constructor => {
            constructor = entry_points.into_iter().map(Into::into).collect();
        }
        EntryPointTypeWrapper::External => {
            external = entry_points.into_iter().map(Into::into).collect();
        }
        EntryPointTypeWrapper::L1Handler => {
            l_1_handler = entry_points.into_iter().map(Into::into).collect();
        }
    });
    DeprecatedEntryPointsByType { constructor, external, l_1_handler }
}

pub fn to_invoke_tx(tx: BroadcastedInvokeTransaction) -> Result<InvokeTransaction> {
    match tx {
        BroadcastedInvokeTransaction::V0(_) => Err(ErrorCode::FailedToReceiveTransaction.into()),
        BroadcastedInvokeTransaction::V1(invoke_tx_v1) => Ok(InvokeTransaction {
            version: 1_u8,
            signature: BoundedVec::try_from(
                invoke_tx_v1.signature.iter().map(|x| H256::from(x.to_bytes_be())).collect::<Vec<H256>>(),
            )
            .map_err(|e| anyhow!("failed to convert signature: {:?}", e))?,
            sender_address: invoke_tx_v1.sender_address.to_bytes_be(),
            nonce: U256::from(invoke_tx_v1.nonce.to_bytes_be()),
            calldata: BoundedVec::try_from(
                invoke_tx_v1.calldata.iter().map(|x| U256::from(x.to_bytes_be())).collect::<Vec<U256>>(),
            )
            .map_err(|e| anyhow!("failed to convert calldata: {:?}", e))?,
            max_fee: U256::from(invoke_tx_v1.max_fee.to_bytes_be()),
        }),
    }
}
