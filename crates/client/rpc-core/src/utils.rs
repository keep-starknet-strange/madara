use std::vec;

use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use mp_starknet::execution::types::ContractClassWrapper;
use starknet_core::types::FieldElement;
use starknet_providers::jsonrpc::models::{ContractClass, EntryPointsByType, SierraContractClass};

/// Returns a `ContractClass` from a `ContractClassWrapper`
// TODO: see https://github.com/keep-starknet-strange/madara/issues/363
pub fn to_rpc_contract_class(_contract_class_wrapped: ContractClassWrapper) -> Result<ContractClass> {
    let entry_points_by_type = EntryPointsByType { constructor: vec![], external: vec![], l1_handler: vec![] };
    let default = SierraContractClass {
        sierra_program: vec![FieldElement::from_dec_str("0").unwrap()],
        contract_class_version: String::from("version"),
        entry_points_by_type,
        abi: String::from(""),
    };
    Ok(ContractClass::Sierra(default))
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
