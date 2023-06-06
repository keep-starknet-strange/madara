use ethers;

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

impl DataAvailability for EthereumDataAvailability {
    /// Publish data to Ethereum.
    /// # Arguments
    /// * `sender_id` - The sender id.
    /// * `data` - The data to publish.
    fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str> {
        self.check_data(data)?;
        // Send data to Ethereum.
        // Check the result
        // Return the result.
        todo!()
    }
}

impl EthereumDataAvailability {
    fn check_data(&self, data: &[u8]) -> Result<(), &str> {
        Ok(())
    }
}
