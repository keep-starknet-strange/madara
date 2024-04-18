use std::sync::Arc;

use async_trait::async_trait;
use zaun_utils::LocalWalletSignerMiddleware;

pub mod eth_bridge;
pub mod starknet_sovereign;
pub mod token_bridge;
pub mod utils;

#[async_trait]
pub trait BridgeDeployable {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self;
}
