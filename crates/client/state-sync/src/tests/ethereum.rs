use std::sync::Arc;
use std::time::Duration;

use ethers::types::{Address, Filter, U256};
use mc_db::L1L2BlockMapping;
use parking_lot::Mutex;

use crate::ethereum::{u256_to_h256, EthereumStateFetcher};
use crate::tests::writer::{create_temp_madara_backend, create_test_client};
use crate::{run, StateFetcher, SyncStatus};

#[tokio::test]
async fn test_fetch_and_decode_state_diff() {
    let contract_address = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e".parse::<Address>().unwrap();
    let verifier_address = "0x5EF3C980Bf970FcE5BbC217835743ea9f0388f4F".parse::<Address>().unwrap();
    let memory_page_address = "0x743789ff2fF82Bfb907009C9911a7dA636D34FA7".parse::<Address>().unwrap();

    let eth_url_list = vec![String::from("https://eth-goerli.g.alchemy.com/v2/nMMxqPTld6cj0DUO-4Qj2cg88Dd1MUhH")];
    let sync_status = Arc::new(Mutex::new(SyncStatus::SYNCING));
    let mut fetcher = EthereumStateFetcher::new(
        contract_address,
        verifier_address,
        memory_page_address,
        eth_url_list,
        28566,
        sync_status,
        1000,
    )
    .unwrap();

    let l1_from = 5854001; // 5789711
    let l2_start = 0;

    let (madara_client, _) = create_test_client();

    let result = fetcher.state_diff(l1_from, l2_start, Arc::new(madara_client)).await.unwrap();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_sync_state_diff_from_l1() {
    let contract_address = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e".parse::<Address>().unwrap();
    let verifier_address = "0xb59D5F625b63fbb04134213A526AA3762555B853".parse::<Address>().unwrap();
    let memory_page_address = "0xdc1534eeBF8CEEe76E31C98F5f5e0F9979476c87".parse::<Address>().unwrap();

    let eth_url_list = vec![String::from("https://eth-goerli.g.alchemy.com/v2/nMMxqPTld6cj0DUO-4Qj2cg88Dd1MUhH")];
    let sync_status = Arc::new(Mutex::new(SyncStatus::SYNCING));
    let fetcher = EthereumStateFetcher::new(
        contract_address,
        verifier_address,
        memory_page_address,
        eth_url_list,
        28566,
        sync_status,
        1000,
    )
    .unwrap();

    let (madara_client, backend) = create_test_client();
    let madara_client = Arc::new(madara_client);
    let madara_backend = create_temp_madara_backend();

    madara_backend
        .meta()
        .write_last_l1_l2_mapping(&L1L2BlockMapping {
            l1_block_hash: Default::default(),
            l1_block_number: 9064757,
            l2_block_hash: Default::default(),
            l2_block_number: 809818,
        })
        .unwrap();

    let madara_backend_clone = madara_backend.clone();
    let task = run(fetcher, madara_backend_clone.clone(), madara_client, backend);

    tokio::spawn(task);

    let checker = async {
        let x: U256 =
            U256::from_dec_str("2409623650734780165222257162509627778194655569248908608781731187419865462224").unwrap();
        let starknet_hash = u256_to_h256(x);
        loop {
            let res = madara_backend.mapping().block_hash(&starknet_hash).unwrap();

            tokio::time::sleep(Duration::from_secs(2)).await;
            if res.is_some() {
                break;
            }
        }
    };

    checker.await
}

#[tokio::test]
async fn test_get_logs_retry() {
    let contract_address = "0xc662c410c0ecf747543f5ba90660f6abebd9c8c4".parse::<Address>().unwrap();
    let verifier_address = "0x47312450B3Ac8b5b8e247a6bB6d523e7605bDb60".parse::<Address>().unwrap();
    let memory_page_address = "0xdc1534eeBF8CEEe76E31C98F5f5e0F9979476c87".parse::<Address>().unwrap();

    let eth_url_list = vec![
        String::from("https://eth.llamarpc.com"),
        String::from("https://eth-goerli.g.alchemy.com/v2/nMMxqPTld6cj0DUO-4Qj2cg88Dd1MUhH"),
    ];

    let sync_status = Arc::new(Mutex::new(SyncStatus::SYNCING));
    let mut client = EthereumStateFetcher::new(
        contract_address,
        verifier_address,
        memory_page_address,
        eth_url_list,
        28566,
        sync_status,
        1000,
    )
    .unwrap();
    let filter = Filter::new().address(contract_address).event("LogStateUpdate(uint256,int256,uint256)");

    let from: u64 = 9064757;
    let to: u64 = 1000001;
    let filter = filter.from_block(from).to_block(to);

    assert!(client.get_logs_retry(&filter).await.is_err());
}
