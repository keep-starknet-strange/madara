//! Data availability module.
pub mod ethereum;

use async_trait::async_trait;

#[async_trait]
pub trait DataAvailability {
    /// Publish data to Ethereum.
    /// # Arguments
    /// * `sender_id` - The sender id.
    /// * `data` - The data to publish.
    async fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str>;
}
