//! Data availability module.
pub mod ethereum;

use async_trait::async_trait;

/// In the short term we will use the lambda service to submit the OS execution trace
pub const TESTNET_SHARP_ADDRESS: &str = "https://testnet.provingservice.io";
pub const STEP_LIMIT: u32 = 1_000_000;

#[async_trait]
pub trait DataAvailability {
    /// Publish data to Ethereum.
    /// # Arguments
    /// * `sender_id` - The sender id.
    /// * `data` - The data to publish.
    async fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str>;
}
