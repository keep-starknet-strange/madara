use serde::Serialize;
use starknet_api::block::BlockNumber;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkHash;
use starknet_client::reader::{StarknetFeederGatewayClient, StarknetReader};
use starknet_client::retry::RetryConfig;
use tokio::join;

const NODE_VERSION: &str = "PAPYRUS-INTEGRATION-TEST-STARKNET-FEEDER-GATEWAY-CLIENT";

#[derive(Serialize)]
// Blocks with API changes to be tested with the get_block function.
struct BlocksForGetBlock {
    // First block, the original definitions.
    first_block: u32,
    // A block with declare transaction. (added in v0.9.0).
    declare_tx: u32,
    // A block with starknet version. (added in v0.9.1).
    starknet_version: u32,
    // A block with declare transaction version 1. (added in v0.10.0).
    // A block with nonce field in transaction. (added in v0.10.0).
    declare_version_1: u32,
    // A block with invoke_function transaction version 1 (added in v0.10.0).
    invoke_version_1: u32,
    // A block with deploy_account transaction. (added in v0.10.1).
    deploy_account: u32,
    // A block with declare transaction version 2. (added in v0.11.0).
    declare_version_2: u32,
}

#[derive(Serialize)]
// Blocks with API changes to be tested with the get_state_update function.
struct BlocksForGetStateUpdate {
    // First block, the original definitions.
    first_block: u32,
    // A state update with 'old_declared_contracts'. (added in v0.9.1).
    old_declared_contracts: u32,
    // A state update with 'nonces'. (added in v0.10.0).
    nonces: u32,
    // A state update with 'declared_classes'. (added in v0.11.0).
    declared_classes: u32,
    // A state update with 'replaced_classes'. (added in v0.11.0).
    replaced_classes: u32,
}

#[derive(Serialize)]
// Class hashes of different versions.
struct ClassHashes {
    // A class definition of Cairo 0 contract.
    cairo_0_class_hash: String,
    // A class definition of Cairo 1 contract. (added in v0.11.0).
    cairo_1_class_hash: String,
}

// Test data for a specific testnet.
struct TestEnvData {
    url: String,
    get_blocks: BlocksForGetBlock,
    get_state_updates: BlocksForGetStateUpdate,
    class_hashes: ClassHashes,
}

fn into_block_number_vec<T: Serialize>(obj: T) -> Vec<BlockNumber> {
    serde_json::to_value(obj)
        .unwrap()
        .as_object()
        .unwrap()
        .values()
        .map(|block_number_json_val| BlockNumber(block_number_json_val.as_u64().unwrap()))
        .collect()
}

#[tokio::test]
#[ignore]
async fn test_integration_testnet() {
    let integration_testnet_data = TestEnvData {
        url: "https://external.integration.starknet.io".to_owned(),
        get_blocks: BlocksForGetBlock {
            first_block: 0,
            declare_tx: 171486,
            starknet_version: 192397,
            declare_version_1: 228224,
            invoke_version_1: 228208,
            deploy_account: 238699,
            declare_version_2: 285182,
        },
        get_state_updates: BlocksForGetStateUpdate {
            first_block: 0,
            old_declared_contracts: 209679,
            nonces: 228155,
            declared_classes: 285182,
            replaced_classes: 0, // No block with this API change yet.
        },
        class_hashes: ClassHashes {
            cairo_0_class_hash: "0x2753ce06a79a9a9c608787a608b424f79c56f465954f1f3a7f6785d575366fb".to_owned(),
            cairo_1_class_hash: "0x2f80a64102b148f7142f1ec14a786ef130e2d4320f2214f4aafebb961e3ab45".to_owned(),
        },
    };
    run(integration_testnet_data).await;
}

#[tokio::test]
#[ignore]
async fn test_alpha_testnet() {
    let alpha_testnet_data = TestEnvData {
        url: "https://alpha4.starknet.io/".to_owned(),
        get_blocks: BlocksForGetBlock {
            first_block: 0,
            declare_tx: 248971,
            starknet_version: 280000,
            declare_version_1: 330039,
            invoke_version_1: 330291,
            deploy_account: 385429,
            declare_version_2: 789048,
        },
        get_state_updates: BlocksForGetStateUpdate {
            first_block: 0,
            old_declared_contracts: 248971,
            nonces: 330039,
            declared_classes: 789048,
            replaced_classes: 788504,
        },
        class_hashes: ClassHashes {
            cairo_0_class_hash: "0x7af612493193c771c1b12f511a8b4d3b0c6d0648242af4680c7cd0d06186f17".to_owned(),
            cairo_1_class_hash: "0x702a9e80c74a214caf0e77326180e72ba3bd3f53dbd5519ede339eb3ae9eed4".to_owned(),
        },
    };
    run(alpha_testnet_data).await;
}

async fn run(test_env_data: TestEnvData) {
    let starknet_client = StarknetFeederGatewayClient::new(
        &test_env_data.url,
        None,
        NODE_VERSION,
        RetryConfig { retry_base_millis: 30, retry_max_delay_millis: 30000, max_retries: 10 },
    )
    .expect("Create new client");

    join!(
        test_get_block(&starknet_client, test_env_data.get_blocks),
        test_get_state_update(&starknet_client, test_env_data.get_state_updates),
        test_class_hash(&starknet_client, test_env_data.class_hashes)
    );
}

// Call get_block on the given list of block_numbers.
async fn test_get_block(starknet_client: &StarknetFeederGatewayClient, block_numbers: BlocksForGetBlock) {
    for block_number in into_block_number_vec(block_numbers) {
        starknet_client.block(block_number).await.unwrap().unwrap();
    }

    // Get the last block.
    starknet_client.latest_block().await.unwrap().unwrap();
    // Not existing block.
    assert!(starknet_client.block(BlockNumber(u64::MAX)).await.unwrap().is_none());
}

// Call get_state_update on the given list of block_numbers.
async fn test_get_state_update(starknet_client: &StarknetFeederGatewayClient, block_numbers: BlocksForGetStateUpdate) {
    for block_number in into_block_number_vec(block_numbers) {
        starknet_client.state_update(block_number).await.unwrap().unwrap();
    }
}

// Call class_by_hash for the given list of class_hashes.
async fn test_class_hash(starknet_client: &StarknetFeederGatewayClient, class_hashes: ClassHashes) {
    let data = serde_json::to_value(class_hashes).unwrap();

    for class_hash_json_val in data.as_object().unwrap().values() {
        let class_hash_val = class_hash_json_val.as_str().unwrap();
        let class_hash = ClassHash(StarkHash::try_from(class_hash_val).unwrap());
        starknet_client.class_by_hash(class_hash).await.unwrap().unwrap();
    }
}
