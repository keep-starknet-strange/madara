extern crate starknet_rpc_test;
use assert_matches::assert_matches;
use blockifier::execution::contract_class::ContractClassV0Inner;
use indexmap::IndexMap;
use parity_scale_codec::Encode;
use rstest::rstest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet_api::core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{DeclareTransactionV0V1, Fee, TransactionHash, TransactionSignature};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{BlockId, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CustomDeclareV0Transaction {
    pub declare_transaction: DeclareTransactionV0V1,
    pub program_vec: Vec<u8>,
    pub entrypoints: IndexMap<EntryPointType, Vec<EntryPoint>>,
    pub abi_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclareV0Result {
    pub txn_hash: TransactionHash,
    pub class_hash: ClassHash,
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_contract(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let unknown_contract_address = FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract Address");

    assert_matches!(
        rpc.get_class_hash_at(BlockId::Number(0), unknown_contract_address,).await,
        Err(StarknetProviderError(StarknetError::ContractNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn declare_v0_contract() -> Result<(), anyhow::Error> {
    let path_to_abi = "../starknet-rpc-test/contracts/proxy.json";

    let contract_artifact: ContractClassV0Inner =
        serde_json::from_reader(std::fs::File::open(path_to_abi).unwrap()).unwrap();

    let contract_abi_artifact: LegacyContractClass =
        serde_json::from_reader(std::fs::File::open(path_to_abi).unwrap()).unwrap();

    let empty_vector_stark_hash: Vec<StarkHash> = Vec::new();
    let empty_array: [u8; 32] = [0; 32];

    let class_info = contract_artifact.clone();

    let program = class_info.program;
    let encoded_p = program.encode();
    let entry_points_by_type = class_info.entry_points_by_type;

    let declare_txn: DeclareTransactionV0V1 = DeclareTransactionV0V1 {
        max_fee: Fee(0),
        signature: TransactionSignature(empty_vector_stark_hash),
        nonce: Nonce(StarkFelt(empty_array)),
        class_hash: ClassHash(StarkHash { 0: contract_abi_artifact.class_hash().unwrap().to_bytes_be() }),
        sender_address: ContractAddress(PatriciaKey(StarkHash { 0: FieldElement::ONE.to_bytes_be() })),
    };
    let abi_length = contract_abi_artifact.abi.len();

    let params: CustomDeclareV0Transaction = CustomDeclareV0Transaction {
        declare_transaction: declare_txn,
        program_vec: encoded_p,
        entrypoints: entry_points_by_type,
        abi_length,
    };

    let json_body = &json!({
        "jsonrpc": "2.0",
        "method": "madara_declareV0",
        "params": [params],
        "id": 4
    });

    let req_client = reqwest::Client::new();
    let raw_txn_rpc = req_client.post("http://localhost:9944").json(json_body).send().await;

    match raw_txn_rpc {
        Ok(val) => {
            let res = val.json::<DeclareV0Result>().await;
            println!("Txn Sent Successfully : {:?}", res);
            println!("Declare Success : {:?}", contract_abi_artifact.class_hash().unwrap());
        }
        Err(err) => {
            println!("Error : Error sending the transaction using RPC");
            println!("{:?}", err);
        }
    }

    Ok(())
}
