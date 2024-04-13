use async_lock::RwLock;
use rstest::fixture;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use url::Url;

use crate::MadaraClient;

pub struct ThreadSafeMadaraClient(RwLock<MadaraClient>);

#[fixture]
#[once]
pub fn madara() -> ThreadSafeMadaraClient {
    ThreadSafeMadaraClient(RwLock::new(MadaraClient::default()))
}

pub fn madara_from(url: Url) -> ThreadSafeMadaraClient {
    ThreadSafeMadaraClient(RwLock::new(MadaraClient::new(url)))
}

impl ThreadSafeMadaraClient {
    pub async fn get_starknet_client(&self) -> JsonRpcClient<HttpTransport> {
        let inner = self.0.read();
        inner.await.get_starknet_client()
    }

    pub async fn write(&self) -> async_lock::RwLockWriteGuard<'_, MadaraClient> {
        self.0.write().await
    }
}
