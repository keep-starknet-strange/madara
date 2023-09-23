use std::{sync::Arc, path::PathBuf};

use mp_transactions::{DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2, InvokeTransactionV0, InvokeTransactionV1};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sp_core::{bounded_vec::BoundedVec, U256};
use blockifier::execution::contract_class::{ContractClass, ContractClassV0, ContractClassV1, ContractClassV1Inner};
use starknet_client::{reader::{objects::transaction::{IntermediateInvokeTransaction, IntermediateDeclareTransaction, DeployAccountTransaction, L1HandlerTransaction, DeployTransaction}, StarknetFeederGatewayClient, StarknetReader, GenericContractClass, ReaderClientError}, RetryConfig};
use starknet_ff::FieldElement;
use starknet_api::{api_core::{PatriciaKey, ClassHash}, stark_felt, hash::StarkFelt};
use std::{env, fs};
use std::fs::File;
use std::io::Write;
use pallet_starknet::genesis_loader::read_contract_class_from_json;


use crate::{RpcConfig, NODE_VERSION};

pub async fn convert_to_contract_class(class_hash: ClassHash) -> Result<ContractClass, String> {
    let rpc_config = RpcConfig::default();

    let retry_config = RetryConfig {
        retry_base_millis: 30,
        retry_max_delay_millis: 30000,
        max_retries: 10,
    };

    let starknet_client = StarknetFeederGatewayClient::new(
        &rpc_config.starknet_url,
        None,
        NODE_VERSION,
        retry_config
    ).map_err(|e| format!("Initialization Error: {:?}", e))?;

    let contract_class = starknet_client.raw_class_by_hash(ClassHash(stark_felt!("0x04b752c44a965256e949c2847354013e539f4cf375a614a584f12e532a4d5225")))
        .await
        .map_err(|e| format!("Error: {:?}", e))?;

    let mut file = File::create("contract_class_output.txt").map_err(|e| format!("File Error: {:?}", e))?;
    write!(file, "{:?}", contract_class).map_err(|e| format!("Write Error: {:?}", e))?;

    let result = read_contract_class_from_json(&contract_class, 1u8);
    Ok(result)
}


pub fn leading_bits(arr: &[u8; 32]) -> U256 {
    let mut count = 0;
    for x in arr {
        let bits = x.leading_zeros();
        count += bits;
        if bits != 8 {
            break;
        }
    }
    U256::from(count)
}

pub fn declare_tx_to_starknet_tx(
    declare_transaction: IntermediateDeclareTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    
    // Convert `IntermediateDeclareTransaction` to `starknet_api::transaction::DeclareTransaction`
    let starknet_declare_tx = starknet_api::transaction::DeclareTransaction::try_from(declare_transaction)?;

    // Convert `starknet_api::transaction::DeclareTransaction` to `mp_transactions::DeclareTransaction`
    let mp_declare_tx = match starknet_declare_tx {
        starknet_api::transaction::DeclareTransaction::V0(inner) => {
            mp_transactions::DeclareTransaction::V0(DeclareTransactionV0::from(inner))
        },
        starknet_api::transaction::DeclareTransaction::V1(inner) => {
            mp_transactions::DeclareTransaction::V1(DeclareTransactionV1::from(inner))
        },
        starknet_api::transaction::DeclareTransaction::V2(inner) => {
            mp_transactions::DeclareTransaction::V2(DeclareTransactionV2::from(inner))
        },
    };

    Ok(mp_transactions::Transaction::Declare(mp_declare_tx))
}

pub fn invoke_tx_to_starknet_tx(
    invoke_transaction: IntermediateInvokeTransaction
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    
    // Try to convert the intermediate representation to the starknet_api representation
    let starknet_invoke_tx = starknet_api::transaction::InvokeTransaction::try_from(invoke_transaction)?;

    // Convert `starknet_api::transaction::InvokeTransaction` to `mp_transactions::InvokeTransaction`
    let mp_invoke_tx = match starknet_invoke_tx {
        starknet_api::transaction::InvokeTransaction::V0(inner) => {
            mp_transactions::InvokeTransaction::V0(mp_transactions::InvokeTransactionV0::from_starknet(inner))
        },
        starknet_api::transaction::InvokeTransaction::V1(inner) => {
            mp_transactions::InvokeTransaction::V1(mp_transactions::InvokeTransactionV1::from_starknet(inner))
        }
    };

    Ok(mp_transactions::Transaction::Invoke(mp_invoke_tx))
}

pub async fn deploy_tx_to_starknet_tx(
    deploy_transaction : starknet_api::transaction::DeployTransaction
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_deploy_tx = mp_transactions::DeployTransaction::from_starknet(deploy_transaction);
    Ok(mp_transactions::Transaction::Deploy(mp_deploy_tx))
}

pub fn deploy_account_tx_to_starknet_tx(
    deploy_account_transaction: starknet_api::transaction::DeployAccountTransaction
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_deploy_account_tx = mp_transactions::DeployAccountTransaction::from_starknet(deploy_account_transaction);
    Ok(mp_transactions::Transaction::DeployAccount(mp_deploy_account_tx))
}


pub fn l1handler_tx_to_starknet_tx(
    l1handler_transaction: starknet_api::transaction::L1HandlerTransaction
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_l1handler_tx = mp_transactions::HandleL1MessageTransaction::from_starknet(l1handler_transaction);
    Ok(mp_transactions::Transaction::L1Handler(mp_l1handler_tx))
}
