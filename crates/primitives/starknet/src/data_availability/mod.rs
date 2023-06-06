//! Data availability module.
pub mod ethereum;

pub trait DataAvailability {
    /// Publish data to Ethereum.
    /// # Arguments
    /// * `sender_id` - The sender id.
    /// * `data` - The data to publish.
    fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str>;
}
