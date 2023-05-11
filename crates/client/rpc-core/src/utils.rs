use std::collections::BTreeMap;

use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use frame_support::storage::bounded_vec::BoundedVec;
use mp_starknet::execution::types::{ContractClassWrapper, EntryPointTypeWrapper, EntryPointWrapper, MaxEntryPoints};

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

/// Returns a hash map of entry point types to entrypoint from deprecated entry point by type
pub fn to_btree_map_entrypoints(
    entries: DeprecatedEntryPointsByType,
) -> BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> {
    let mut entry_points_by_type: BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> =
        BTreeMap::new();
    // We can unwrap safely as we already checked the length of the vectors
    entry_points_by_type.insert(
        EntryPointTypeWrapper::Constructor,
        BoundedVec::try_from(
            entries.constructor.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>(),
        )
        .unwrap(),
    );
    entry_points_by_type.insert(
        EntryPointTypeWrapper::External,
        BoundedVec::try_from(entries.external.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>())
            .unwrap(),
    );
    entry_points_by_type.insert(
        EntryPointTypeWrapper::L1Handler,
        BoundedVec::try_from(
            entries.l_1_handler.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>(),
        )
        .unwrap(),
    );
    entry_points_by_type
}

/// Removes the "0x" prefix from a given hexadecimal string and pads it with 0s
#[inline(always)]
pub fn format_hex(input: &str) -> String {
    format!("{:0>64}", input.strip_prefix("0x").unwrap_or(input))
}
