use rstest::fixture;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeclareTransactionV1, BroadcastedTransaction,
    CompressedLegacyContractClass,
};
use starknet_ff::FieldElement;

use crate::{ExecutionStrategy, MadaraClient};
#[fixture]
pub async fn madara() -> MadaraClient {
    MadaraClient::new(ExecutionStrategy::Native).await
}
