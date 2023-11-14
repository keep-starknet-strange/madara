use std::time::Duration;

use primitive_types::H160;
use tokio::time;

use crate::ethereum::{EthereumApi, EthereumChain, EthereumStateUpdate};
use crate::l1::{sync_from_l1_loop, L1SyncContext};

// Create a mock implementation of the EthereumApi trait for testing
#[derive(Clone)]
struct MockEthereumApi;

#[tokio::test]
async fn test_sync_from_l1_loop() {
    // Set up test data and parameters
    let ethereum = MockEthereumApi; // Use your actual mock implementation
    let core_address = H160::default(); // Provide a valid address for testing
    let poll_interval = Duration::from_secs(1); // Adjust as needed for testing

    // Create the L1SyncContext for testing
    let context = L1SyncContext { ethereum, core_address, poll_interval };

    // Run the sync_from_l1_loop function in a separate task
    let sync_task = tokio::spawn(sync_from_l1_loop(context));

    // Add assertions or checks based on your requirements
    // For example, you can use the tokio::time::sleep to wait for a specific duration and then check
    // the state.
    time::sleep(Duration::from_secs(5)).await;

    // You might want to cancel the task at some point, depending on your testing needs.
    sync_task.abort(); // Note: Requires tokio 1.3 or later

    // Use assertions to check the results or side effects of the sync_from_l1_loop function
    // assert!(some_condition, "Some meaningful error message");
}

// Implement the EthereumApi trait for the MockEthereumApi
#[async_trait::async_trait]
impl EthereumApi for MockEthereumApi {
    async fn get_starknet_state(&self, _address: &H160) -> anyhow::Result<EthereumStateUpdate> {
        // Implement a mock response for get_starknet_state
        Ok(EthereumStateUpdate {
            state_root: Default::default(),
            block_number: Default::default(),
            block_hash: Default::default(),
        })
    }

    async fn get_chain(&self) -> anyhow::Result<EthereumChain> {
        // Implement a mock response for get_chain
        Ok(EthereumChain::Mainnet)
    }
}
