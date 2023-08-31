use rstest::fixture;

use crate::{ExecutionStrategy, MadaraClient};

#[fixture]
pub async fn madara() -> MadaraClient {
    MadaraClient::new(ExecutionStrategy::Native).await
}
