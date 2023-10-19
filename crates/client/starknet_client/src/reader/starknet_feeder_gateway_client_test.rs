use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;

use assert_matches::assert_matches;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use indexmap::indexmap;
use mockito::mock;
use pretty_assertions::assert_eq;
use starknet_api::block::BlockNumber;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::deprecated_contract_class::{
    ContractClass as DeprecatedContractClass, ContractClassAbiEntry, EntryPoint as DeprecatedEntryPoint,
    EntryPointOffset, EntryPointType as DeprecatedEntryPointType, FunctionAbiEntry, Program, TypedParameter,
};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::{EntryPoint, EntryPointType, FunctionIndex};
use starknet_api::transaction::{Fee, TransactionHash, TransactionSignature, TransactionVersion};
use starknet_api::{patricia_key, stark_felt};

use super::objects::state::StateUpdate;
use super::objects::transaction::IntermediateDeclareTransaction;
use super::{
    Block, ContractClass, GenericContractClass, ReaderClientError, ReaderClientResult, StarknetFeederGatewayClient,
    StarknetReader, BLOCK_NUMBER_QUERY, CLASS_HASH_QUERY, GET_BLOCK_URL, GET_STATE_UPDATE_URL,
};
use crate::test_utils::read_resource::read_resource_file;
use crate::test_utils::retry::get_test_config;

const NODE_VERSION: &str = "NODE VERSION";

#[test]
fn new_urls() {
    let url_base_str = "https://url";
    let starknet_client =
        StarknetFeederGatewayClient::new(url_base_str, None, NODE_VERSION, get_test_config()).unwrap();
    assert_eq!(starknet_client.urls.get_block.as_str(), url_base_str.to_string() + "/" + GET_BLOCK_URL);
    assert_eq!(starknet_client.urls.get_state_update.as_str(), url_base_str.to_string() + "/" + GET_STATE_UPDATE_URL);
}

#[tokio::test]
async fn get_block_number() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    // There are blocks in Starknet.
    let mock_block = mock("GET", "/feeder_gateway/get_block?blockNumber=latest")
        .with_status(200)
        .with_body(read_resource_file("reader/block.json"))
        .create();
    let latest_block = starknet_client.latest_block().await.unwrap();
    mock_block.assert();
    assert_eq!(latest_block.unwrap().block_number, BlockNumber(273466));

    // There are no blocks in Starknet.
    let body = r#"{"code": "StarknetErrorCode.BLOCK_NOT_FOUND", "message": "Block number -1 was not found."}"#;
    let mock_no_block =
        mock("GET", "/feeder_gateway/get_block?blockNumber=latest").with_status(400).with_body(body).create();
    let latest_block = starknet_client.latest_block().await.unwrap();
    mock_no_block.assert();
    assert!(latest_block.is_none());
}

#[tokio::test]
async fn declare_tx_serde() {
    let declare_tx = IntermediateDeclareTransaction {
        class_hash: ClassHash(stark_felt!("0x7319e2f01b0947afd86c0bb0e95029551b32f6dc192c47b2e8b08415eebbc25")),
        compiled_class_hash: None,
        sender_address: ContractAddress(patricia_key!("0x1")),
        nonce: Nonce(stark_felt!("0x0")),
        max_fee: Fee(0),
        version: TransactionVersion(stark_felt!("0x1")),
        transaction_hash: TransactionHash(stark_felt!(
            "0x2f2ef64daffdc72bf33b34ad024891691b8eb1d0ab70cc7f8fb71f6fd5e1f22"
        )),
        signature: TransactionSignature(vec![]),
    };
    let raw_declare_tx = serde_json::to_string(&declare_tx).unwrap();
    assert_eq!(declare_tx, serde_json::from_str(&raw_declare_tx).unwrap());
}

#[tokio::test]
async fn state_update() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let raw_state_update = read_resource_file("reader/block_state_update.json");
    let mock_state_update = mock("GET", &format!("/feeder_gateway/get_state_update?{BLOCK_NUMBER_QUERY}=123456")[..])
        .with_status(200)
        .with_body(&raw_state_update)
        .create();
    let state_update = starknet_client.state_update(BlockNumber(123456)).await.unwrap();
    mock_state_update.assert();
    let expected_state_update: StateUpdate = serde_json::from_str(&raw_state_update).unwrap();
    assert_eq!(state_update.unwrap(), expected_state_update);

    let body = r#"{"code": "StarknetErrorCode.BLOCK_NOT_FOUND", "message": "Block number -1 was not found."}"#;
    let mock_no_block = mock("GET", &format!("/feeder_gateway/get_state_update?{BLOCK_NUMBER_QUERY}=999999")[..])
        .with_status(400)
        .with_body(body)
        .create();
    let state_update = starknet_client.state_update(BlockNumber(999999)).await.unwrap();
    assert!(state_update.is_none());
    mock_no_block.assert();
}

#[tokio::test]
async fn serialization_precision() {
    let input = "{\"value\":244116128358498188146337218061232635775543270890529169229936851982759783745}";
    let serialized = serde_json::from_str::<serde_json::Value>(input).unwrap();
    let deserialized = serde_json::to_string(&serialized).unwrap();
    assert_eq!(input, deserialized);
}

#[tokio::test]
async fn contract_class() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let expected_contract_class = ContractClass {
        sierra_program: vec![
            stark_felt!("0x302e312e30"),
            stark_felt!("0x1c"),
            stark_felt!("0x52616e6765436865636b"),
        ],
        entry_points_by_type: HashMap::from([(
            EntryPointType::External,
            vec! [EntryPoint {
                function_idx: FunctionIndex(0),
                selector: EntryPointSelector(stark_felt!(
                    "0x22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658"
                )),
            }],
        ),
        (EntryPointType::Constructor, vec![]),
        (EntryPointType::L1Handler, vec![]),
        ]),
        contract_class_version: String::from("0.1.0"),
        abi: String::from("[\n  {\n    \"type\": \"function\",\n    \"name\": \"test\",\n    \"inputs\": [\n      {\n        \"name\": \"arg\",\n        \"ty\": \"core::felt\"\n      },\n      {\n        \"name\": \"arg1\",\n        \"ty\": \"core::felt\"\n      },\n      {\n        \"name\": \"arg2\",\n        \"ty\": \"core::felt\"\n      }\n    ],\n    \"output_ty\": \"core::felt\",\n    \"state_mutability\": \"external\"\n  },\n  {\n    \"type\": \"function\",\n    \"name\": \"empty\",\n    \"inputs\": [],\n    \"output_ty\": \"()\",\n    \"state_mutability\": \"external\"\n  },\n  {\n    \"type\": \"function\",\n    \"name\": \"call_foo\",\n    \"inputs\": [\n      {\n        \"name\": \"a\",\n        \"ty\": \"core::integer::u128\"\n      }\n    ],\n    \"output_ty\": \"core::integer::u128\",\n    \"state_mutability\": \"external\"\n  }\n]"),
    };

    let mock_by_hash = mock(
        "GET",
        &format!(
            "/feeder_gateway/get_class_by_hash?\
             {CLASS_HASH_QUERY}=0x4e70b19333ae94bd958625f7b61ce9eec631653597e68645e13780061b2136c"
        )[..],
    )
    .with_status(200)
    .with_body(read_resource_file("reader/contract_class.json"))
    .create();
    let contract_class = starknet_client
        .class_by_hash(ClassHash(stark_felt!("0x4e70b19333ae94bd958625f7b61ce9eec631653597e68645e13780061b2136c")))
        .await
        .unwrap()
        .unwrap();

    let contract_class = match contract_class {
        GenericContractClass::Cairo1ContractClass(class) => class,
        _ => unreachable!("Expecting Cairo0ContractClass."),
    };
    mock_by_hash.assert();
    assert_eq!(contract_class, expected_contract_class);
}

#[tokio::test]
async fn deprecated_contract_class() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let expected_contract_class = DeprecatedContractClass {
        abi: Some(vec![ContractClassAbiEntry::Constructor(FunctionAbiEntry {
            name: "constructor".to_string(),
            inputs: vec![TypedParameter { name: "implementation".to_string(), r#type: "felt".to_string() }],
            outputs: vec![],
            state_mutability: None,
        })]),
        program: Program {
            attributes: serde_json::Value::Array(vec![serde_json::json!(1234)]),
            builtins: serde_json::Value::Array(Vec::new()),
            compiler_version: serde_json::Value::Null,
            data: serde_json::Value::Array(vec![
                serde_json::Value::String("0x20780017fff7ffd".to_string()),
                serde_json::Value::String("0x4".to_string()),
                serde_json::Value::String("0x400780017fff7ffd".to_string()),
            ]),
            debug_info: serde_json::Value::Null,
            hints: serde_json::Value::Object(serde_json::Map::new()),
            identifiers: serde_json::Value::Object(serde_json::Map::new()),
            main_scope: serde_json::Value::String("__main__".to_string()),
            prime: serde_json::Value::String(
                "0x800000000000011000000000000000000000000000000000000000000000001".to_string(),
            ),
            reference_manager: serde_json::Value::Object(serde_json::Map::new()),
        },
        entry_points_by_type: HashMap::from([
            (DeprecatedEntryPointType::L1Handler, vec![]),
            (
                DeprecatedEntryPointType::Constructor,
                vec![DeprecatedEntryPoint {
                    selector: EntryPointSelector(stark_felt!(
                        "0x028ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194"
                    )),
                    offset: EntryPointOffset(62),
                }],
            ),
            (
                DeprecatedEntryPointType::External,
                vec![DeprecatedEntryPoint {
                    selector: EntryPointSelector(stark_felt!(
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                    )),
                    offset: EntryPointOffset(86),
                }],
            ),
        ]),
    };
    let mock_by_hash = mock(
        "GET",
        &format!(
            "/feeder_gateway/get_class_by_hash?\
             {CLASS_HASH_QUERY}=0x7af612493193c771c1b12f511a8b4d3b0c6d0648242af4680c7cd0d06186f17"
        )[..],
    )
    .with_status(200)
    .with_body(read_resource_file("reader/deprecated_contract_class.json"))
    .create();
    let contract_class = starknet_client
        .class_by_hash(ClassHash(stark_felt!("0x7af612493193c771c1b12f511a8b4d3b0c6d0648242af4680c7cd0d06186f17")))
        .await
        .unwrap()
        .unwrap();
    let contract_class = match contract_class {
        GenericContractClass::Cairo0ContractClass(class) => class,
        _ => unreachable!("Expecting deprecated contract class."),
    };
    mock_by_hash.assert();
    assert_eq!(contract_class, expected_contract_class);

    // Undeclared class.
    let body = r#"{"code": "StarknetErrorCode.UNDECLARED_CLASS", "message": "Class with hash 0x7 is not declared."}"#;
    let mock_by_hash = mock("GET", &format!("/feeder_gateway/get_class_by_hash?{CLASS_HASH_QUERY}=0x7")[..])
        .with_status(400)
        .with_body(body)
        .create();
    let class = starknet_client.class_by_hash(ClassHash(stark_felt!("0x7"))).await.unwrap();
    mock_by_hash.assert();
    assert!(class.is_none());
}

#[tokio::test]
async fn get_block() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let raw_block = read_resource_file("reader/block.json");
    let mock_block = mock("GET", &format!("/feeder_gateway/get_block?{BLOCK_NUMBER_QUERY}=20")[..])
        .with_status(200)
        .with_body(&raw_block)
        .create();
    let block = starknet_client.block(BlockNumber(20)).await.unwrap().unwrap();
    println!("{:?}", block);
    mock_block.assert();
    let expected_block: Block = serde_json::from_str(&raw_block).unwrap();
    assert_eq!(block, expected_block);

    // Non-existing block.
    let body = r#"{"code": "StarknetErrorCode.BLOCK_NOT_FOUND", "message": "Block 9999999999 was not found."}"#;
    let mock_no_block = mock("GET", &format!("/feeder_gateway/get_block?{BLOCK_NUMBER_QUERY}=9999999999")[..])
        .with_status(400)
        .with_body(body)
        .create();
    let block = starknet_client.block(BlockNumber(9999999999)).await.unwrap();
    mock_no_block.assert();
    assert!(block.is_none());
}

#[tokio::test]
async fn compiled_class_by_hash() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let raw_casm_contract_class = read_resource_file("reader/casm_contract_class.json");
    let mock_casm_contract_class =
        mock("GET", &format!("/feeder_gateway/get_compiled_class_by_class_hash?{CLASS_HASH_QUERY}=0x7")[..])
            .with_status(200)
            .with_body(&raw_casm_contract_class)
            .create();
    let casm_contract_class =
        starknet_client.compiled_class_by_hash(ClassHash(stark_felt!("0x7"))).await.unwrap().unwrap();
    mock_casm_contract_class.assert();
    let expected_casm_contract_class: CasmContractClass = serde_json::from_str(&raw_casm_contract_class).unwrap();
    assert_eq!(casm_contract_class, expected_casm_contract_class);

    let body = r#"{"code": "StarknetErrorCode.UNDECLARED_CLASS", "message": "Class with hash 0x7 is not declared."}"#;
    let mock_undeclared =
        mock("GET", &format!("/feeder_gateway/get_compiled_class_by_class_hash?{CLASS_HASH_QUERY}=0x0")[..])
            .with_status(400)
            .with_body(body)
            .create();
    let class = starknet_client.compiled_class_by_hash(ClassHash(stark_felt!("0x0"))).await.unwrap();
    mock_undeclared.assert();
    assert!(class.is_none());
}

#[tokio::test]
async fn state_update_with_empty_storage_diff() {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let mut state_update = StateUpdate::default();
    state_update.state_diff.storage_diffs = indexmap!(ContractAddress::default() => vec![]);

    let mock = mock("GET", &format!("/feeder_gateway/get_state_update?{BLOCK_NUMBER_QUERY}=123456")[..])
        .with_status(200)
        .with_body(serde_json::to_string(&state_update).unwrap())
        .create();
    let state_update = starknet_client.state_update(BlockNumber(123456)).await.unwrap().unwrap();
    mock.assert();
    assert!(state_update.state_diff.storage_diffs.is_empty());
}

async fn test_unserializable<
    Output: Send + Debug,
    Fut: Future<Output = ReaderClientResult<Output>>,
    F: FnOnce(StarknetFeederGatewayClient) -> Fut,
>(
    url_suffix: &str,
    call_method: F,
) {
    let starknet_client =
        StarknetFeederGatewayClient::new(&mockito::server_url(), None, NODE_VERSION, get_test_config()).unwrap();
    let body = "body";
    let mock = mock("GET", url_suffix).with_status(200).with_body(body).create();
    let error = call_method(starknet_client).await.unwrap_err();
    mock.assert();
    assert_matches!(error, ReaderClientError::SerdeError(_));
}

#[tokio::test]
async fn latest_block_unserializable() {
    test_unserializable("/feeder_gateway/get_block?blockNumber=latest", |starknet_client| async move {
        starknet_client.latest_block().await
    })
    .await
}

#[tokio::test]
async fn block_unserializable() {
    test_unserializable("/feeder_gateway/get_block?blockNumber=20", |starknet_client| async move {
        starknet_client.block(BlockNumber(20)).await
    })
    .await
}

#[tokio::test]
async fn class_by_hash_unserializable() {
    test_unserializable(
        &format!("/feeder_gateway/get_class_by_hash?{CLASS_HASH_QUERY}=0x1")[..],
        |starknet_client| async move { starknet_client.class_by_hash(ClassHash(stark_felt!("0x1"))).await },
    )
    .await
}

#[tokio::test]
async fn state_update_unserializable() {
    test_unserializable(
        &format!("/feeder_gateway/get_state_update?{BLOCK_NUMBER_QUERY}=123456")[..],
        |starknet_client| async move { starknet_client.state_update(BlockNumber(123456)).await },
    )
    .await
}

#[tokio::test]
async fn compiled_class_by_hash_unserializable() {
    test_unserializable(
        &format!("/feeder_gateway/get_compiled_class_by_class_hash?{CLASS_HASH_QUERY}=0x7")[..],
        |starknet_client| async move { starknet_client.compiled_class_by_hash(ClassHash(stark_felt!("0x7"))).await },
    )
    .await
}
