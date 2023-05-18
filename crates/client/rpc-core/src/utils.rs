use std::vec;

use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use mp_starknet::execution::types::ContractClassWrapper;
use mp_starknet::transaction::types::InvokeTransaction;
use sp_core::{H256, U256};
use sp_runtime::BoundedVec;
use starknet_core::types::FieldElement;
use starknet_providers::jsonrpc::models::{
    BroadcastedInvokeTransaction, ContractClass, EntryPointsByType, ErrorCode, SierraContractClass,
};

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
