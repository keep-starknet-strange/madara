//! This module contains client that can request changes to [`Starknet`].
//!
//! [`Starknet`]: https://starknet.io/

pub mod objects;

#[cfg(test)]
mod starknet_gateway_client_test;

use async_trait::async_trait;
#[cfg(any(feature = "testing", test))]
use mockall::automock;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

use crate::writer::objects::response::{DeclareResponse, DeployAccountResponse, InvokeResponse};
use crate::writer::objects::transaction::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction};
use crate::{ClientCreationError, ClientError, RetryConfig, StarknetClient};

/// Errors that may be returned from a writer client.
#[derive(thiserror::Error, Debug)]
pub enum WriterClientError {
    /// A client error representing errors from the base StarknetClient.
    #[error(transparent)]
    ClientError(#[from] ClientError),
    /// A client error representing deserialization errors.
    /// Note: [`ClientError`] contains SerdeError as well. The difference is that this variant is
    /// responsible for serde errors coming from [`StarknetWriter`] and ClientError::SerdeError
    /// is responsible for serde errors coming from StarknetClient.
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

pub type WriterClientResult<T> = Result<T, WriterClientError>;

/// A trait describing an object that can communicate with [`Starknet`] and make changes to it.
///
/// [`Starknet`]: https://starknet.io/
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait StarknetWriter: Sync + Send + 'static {
    /// Add an invoke transaction to [`Starknet`].
    ///
    /// [`Starknet`]: https://starknet.io/
    async fn add_invoke_transaction(&self, tx: &InvokeTransaction) -> WriterClientResult<InvokeResponse>;

    /// Add a declare transaction to [`Starknet`].
    ///
    /// [`Starknet`]: https://starknet.io/
    async fn add_declare_transaction(&self, tx: &DeclareTransaction) -> WriterClientResult<DeclareResponse>;

    /// Add a deploy account transaction to [`Starknet`].
    ///
    /// [`Starknet`]: https://starknet.io/
    async fn add_deploy_account_transaction(
        &self,
        tx: &DeployAccountTransaction,
    ) -> WriterClientResult<DeployAccountResponse>;
}

const ADD_TRANSACTION_URL_SUFFIX: &str = "gateway/add_transaction";

/// A client for the [`Starknet`] gateway.
///
/// [`Starknet`]: https://starknet.io/
pub struct StarknetGatewayClient {
    add_transaction_url: Url,
    client: StarknetClient,
}

#[async_trait]
impl StarknetWriter for StarknetGatewayClient {
    #[instrument(skip(self), level = "debug")]
    async fn add_invoke_transaction(&self, tx: &InvokeTransaction) -> WriterClientResult<InvokeResponse> {
        self.add_transaction(&tx).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn add_deploy_account_transaction(
        &self,
        tx: &DeployAccountTransaction,
    ) -> WriterClientResult<DeployAccountResponse> {
        self.add_transaction(&tx).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn add_declare_transaction(&self, tx: &DeclareTransaction) -> WriterClientResult<DeclareResponse> {
        self.add_transaction(&tx).await
    }
}

impl StarknetGatewayClient {
    pub fn new(
        starknet_url: &str,
        node_version: &'static str,
        retry_config: RetryConfig,
    ) -> Result<Self, ClientCreationError> {
        Ok(StarknetGatewayClient {
            add_transaction_url: Url::parse(starknet_url)?.join(ADD_TRANSACTION_URL_SUFFIX)?,
            client: StarknetClient::new(None, node_version, retry_config)?,
        })
    }

    async fn add_transaction<Transaction: Serialize, Response: for<'a> Deserialize<'a>>(
        &self,
        tx: &Transaction,
    ) -> WriterClientResult<Response> {
        let response: String = self
            .client
            .request_with_retry(
                self.client.internal_client.post(self.add_transaction_url.clone()).body(serde_json::to_string(&tx)?),
            )
            .await?;
        Ok(serde_json::from_str::<Response>(&response)?)
    }
}
