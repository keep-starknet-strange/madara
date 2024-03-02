use mc_db::DbError;
use sp_api::ApiError;
use url::ParseError;

use crate::contract::L1EventToTransactionError;

#[derive(thiserror::Error, Debug)]
pub enum L1MessagesWorkerError<PE> {
    #[error("Failed to initialize L1 Messages Worker based on provided Config: `{0}`")]
    ConfigError(#[from] ParseError),
    #[error("Failed to convert transaction via Runtime API: `{0}`")]
    ConvertTransactionRuntimeApiError(ApiError),
    #[error("Madara Messaging DB Error: `{0}`")]
    DatabaseError(#[from] DbError),
    #[error("Message from L1 has been already processed, nonce: `{0}`")]
    L1MessageAlreadyProcessed(u64),
    #[error("Failed to use Runtime API: `{0}`")]
    RuntimeApiError(ApiError),
    #[error("Failed to submit transaction into Transaction Pool")]
    SubmitTxError(#[source] PE),
    #[error("Failed to convert L1 Message into Fee")]
    ToFeeError,
    #[error("Failed to convert L1 Message into L2 Transaction: `{0}`")]
    ToTransactionError(#[from] L1EventToTransactionError),
    #[error("Ethereum client error: {0}")]
    EthereumClient(#[from] mc_eth_client::error::Error),
}
