use std::vec;

use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::{DeployAccountTransaction, InvokeTransaction};
use sp_core::U256;
use sp_runtime::BoundedVec;
use starknet_api::api_core::{calculate_contract_address, ClassHash, ContractAddress as StarknetContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_core::types::FieldElement;
use starknet_providers::jsonrpc::models::{
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, ContractClass, EntryPointsByType, ErrorCode,
    SierraContractClass,
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
                invoke_tx_v1.signature.iter().map(|x| (*x).into()).collect::<Vec<Felt252Wrapper>>(),
            )
            .map_err(|e| anyhow!("failed to convert signature: {:?}", e))?,

            // Safe to unwrap, starknet-core already parsed the FieldElement with jsonrpsee.
            sender_address: Felt252Wrapper::try_from(&invoke_tx_v1.sender_address.to_bytes_be()).unwrap(),
            nonce: U256::from(invoke_tx_v1.nonce.to_bytes_be()),
            calldata: BoundedVec::try_from(
                invoke_tx_v1.calldata.iter().map(|x| (*x).into()).collect::<Vec<Felt252Wrapper>>(),
            )
            .map_err(|e| anyhow!("failed to convert calldata: {:?}", e))?,
            max_fee: U256::from(invoke_tx_v1.max_fee.to_bytes_be()),
        }),
    }
}

pub fn to_deploy_account_tx(tx: BroadcastedDeployAccountTransaction) -> Result<DeployAccountTransaction> {
    let version = tx.version;
    let version: u8 =
        version.try_into().map_err(|e| anyhow!("failed to convert version '{}' to u8: {e}", tx.version))?;

    let contract_address_salt = tx.contract_address_salt.to_bytes_be();

    let account_class_hash = tx.class_hash.to_bytes_be();

    let calldata =
        tx.constructor_calldata.iter().filter_map(|f| StarkFelt::new(f.to_bytes_be()).ok()).collect::<Vec<_>>();

    let signature = tx
        .signature
        .iter()
        .map(|f| (*f).into())
        .collect::<Vec<Felt252Wrapper>>()
        .try_into()
        .map_err(|_| anyhow!("failed to bound signatures Vec<H256> by MaxArraySize"))?;

    let sender_address = Felt252Wrapper::try_from(
        &calculate_contract_address(
            ContractAddressSalt(StarkFelt(contract_address_salt)),
            ClassHash(StarkFelt(account_class_hash)),
            &Calldata(calldata.into()),
            StarknetContractAddress::default(),
        )
        .map_err(|e| anyhow!("Failed to calculate contract address: {e}"))?
        .0
        .0
        .0,
    )
    .unwrap(); // Ok to unwrap, starknet-core parsed type.

    let calldata = tx
        .constructor_calldata
        .iter()
        .map(|f| (*f).into())
        .collect::<Vec<Felt252Wrapper>>()
        .try_into()
        .map_err(|_| anyhow!("failed to bound calldata Vec<U256> by MaxArraySize"))?;

    let nonce = U256::from(tx.nonce.to_bytes_be());
    let max_fee = U256::from(tx.max_fee.to_bytes_be());

    Ok(DeployAccountTransaction {
        version,
        sender_address,
        calldata,
        salt: U256::from(contract_address_salt),
        signature,
        account_class_hash: Felt252Wrapper::try_from(&account_class_hash).unwrap(), /* Ok to unwrap, starknet-core
                                                                                     * parsed type. */
        nonce,
        max_fee,
    })
}
