use async_trait::async_trait;
use ethers::prelude::*;

use super::DataAvailability;
/// Ethereum data availability configuration.
/// * TODO
/// - use types ethers.rs
pub struct EthereumDataAvailabilityConfig {
    pub sender_address: [u8; 20],
    pub execution_engine_rpc_url: String,
}

/// Ethereum data availability.
pub struct EthereumDataAvailability {
    pub config: EthereumDataAvailabilityConfig,
}

#[async_trait]
impl DataAvailability for EthereumDataAvailability {
    /// Publish data to Ethereum.
    /// # Arguments
    /// * `sender_id` - The sender id.
    /// * `data` - The data to publish.
    async fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str> {
        self.check_data(data)?;
        // Send data to Ethereum.
        // Check the result
        // Return the result.
            /**
         Updates the state of the StarkNet, based on a proof of the
        StarkNet OS that the state transition is valid.

        Arguments:
            programOutput - The main part of the StarkNet OS program output.
            data_availability_fact - An encoding of the on-chain data associated
            with the 'programOutput'.
        */
        
        abigen!(
            STARKNET,
            r#"[
                function updateState(
                    uint256[] calldata programOutput,
                    uint256 onchainDataHash,
                    uint256 onchainDataSize
                ) external onlyOperator
            ]"#,
        );

        // let provider = Provider::<Http>::try_from(RPC_URL)?;
        // let client = Arc::new(provider);
        todo!()
    }
}

impl EthereumDataAvailability {
    fn check_data(&self, data: &[u8]) -> Result<(), &str> {
        Ok(())
    }
}
