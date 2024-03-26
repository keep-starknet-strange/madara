#![feature(assert_matches)]

use std::cell::Cell;
use std::fmt::Debug;

use anyhow::anyhow;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde_json::json;
use starknet_accounts::{
    Account, AccountDeployment, AccountError, AccountFactoryError, Declaration, Execution, LegacyDeclaration,
    OpenZeppelinAccountFactory, SingleOwnerAccount,
};
use starknet_core::types::{DeclareTransactionResult, DeployAccountTransactionResult, InvokeTransactionResult};
use starknet_providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_providers::Provider;
use starknet_signers::local_wallet::SignError;
use starknet_signers::LocalWallet;
use url::Url;

/// Constants (addresses, contracts...)
pub mod constants;
/// Starknet related utilities
pub mod utils;

pub mod fixtures;

const NODE_RPC_URL: &str = "http://localhost:9944";

type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub type RpcOzAccountFactory<'a> = OpenZeppelinAccountFactory<LocalWallet, &'a JsonRpcClient<HttpTransport>>;
pub type TransactionExecution<'a> = Execution<'a, RpcAccount<'a>>;
type TransactionDeclaration<'a> = Declaration<'a, RpcAccount<'a>>;
type TransactionLegacyDeclaration<'a> = LegacyDeclaration<'a, RpcAccount<'a>>;
type TransactionAccountDeployment<'a> = AccountDeployment<'a, RpcOzAccountFactory<'a>>;
type StarknetAccountError =
    AccountError<<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> as Account>::SignError>;

pub enum Transaction<'a> {
    Execution(TransactionExecution<'a>),
    Declaration(TransactionDeclaration<'a>),
    LegacyDeclaration(TransactionLegacyDeclaration<'a>),
    AccountDeployment(TransactionAccountDeployment<'a>),
}

#[derive(Debug)]
pub enum TransactionResult {
    Execution(InvokeTransactionResult),
    Declaration(DeclareTransactionResult),
    AccountDeployment(DeployAccountTransactionResult),
}

#[derive(thiserror::Error, Debug)]
pub enum SendTransactionError {
    #[error(transparent)]
    AccountError(StarknetAccountError),
    #[error(transparent)]
    AccountFactoryError(AccountFactoryError<SignError>),
}

impl Transaction<'_> {
    pub async fn send(&self) -> Result<TransactionResult, SendTransactionError> {
        match self {
            Transaction::Execution(execution) => {
                execution.send().await.map(TransactionResult::Execution).map_err(SendTransactionError::AccountError)
            }
            Transaction::Declaration(declaration) => {
                declaration.send().await.map(TransactionResult::Declaration).map_err(SendTransactionError::AccountError)
            }
            Transaction::LegacyDeclaration(declaration) => {
                declaration.send().await.map(TransactionResult::Declaration).map_err(SendTransactionError::AccountError)
            }
            Transaction::AccountDeployment(deployment) => deployment
                .send()
                .await
                .map(TransactionResult::AccountDeployment)
                .map_err(SendTransactionError::AccountFactoryError),
        }
    }
}

#[derive(Debug)]
/// A wrapper over the Madara process handle, reqwest client and request counter
pub struct MadaraClient {
    rpc_request_count: Cell<usize>,
    url: Url,
}

impl Default for MadaraClient {
    fn default() -> Self {
        let url = Url::parse(NODE_RPC_URL).expect("Invalid JSONRPC Url");
        MadaraClient::new(url)
    }
}

impl MadaraClient {
    pub fn new(url: Url) -> Self {
        Self { url, rpc_request_count: Default::default() }
    }

    pub async fn run_to_block(&mut self, target_block: u64) -> anyhow::Result<()> {
        let mut current_block = self.get_starknet_client().block_number().await?;

        if current_block >= target_block {
            return Err(anyhow!("target_block must be in the future"));
        }

        while current_block < target_block {
            self.create_empty_block().await?;
            current_block += 1;
        }

        Ok(())
    }

    pub async fn create_n_blocks(&mut self, mut n: u64) -> anyhow::Result<()> {
        while n > 0 {
            self.create_empty_block().await?;
            n -= 1;
        }

        Ok(())
    }

    async fn call_rpc(&self, mut body: serde_json::Value) -> reqwest::Result<Response> {
        let body = body.as_object_mut().expect("the body should be an object");
        body.insert("id".to_string(), self.rpc_request_count.get().into());
        body.insert("jsonrpc".to_string(), "2.0".into());

        let body = serde_json::to_string(&body).expect("the json body must be serializable");

        let response = Client::new()
            .post(self.url.clone())
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .body(body)
            .send()
            .await?;

        // Increment rpc_request_count
        let previous = self.rpc_request_count.get();
        self.rpc_request_count.set(previous + 1);

        Ok(response)
    }

    pub fn get_starknet_client(&self) -> JsonRpcClient<HttpTransport> {
        JsonRpcClient::new(HttpTransport::new(self.url.clone()))
    }

    pub async fn create_empty_block(&mut self) -> anyhow::Result<()> {
        self.do_create_block(true, true).await
    }

    pub async fn create_block_with_pending_txs(&mut self) -> anyhow::Result<()> {
        self.do_create_block(false, true).await
    }

    async fn do_create_block(&mut self, empty: bool, finalize: bool) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [empty, finalize],
        });

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn create_block_with_txs(
        &mut self,
        transactions: Vec<Transaction<'_>>,
    ) -> anyhow::Result<Vec<Result<TransactionResult, SendTransactionError>>> {
        let mut results = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let result = tx.send().await;
            results.push(result);
        }

        self.do_create_block(false, false).await?;
        Ok(results)
    }

    pub async fn submit_txs(
        &mut self,
        transactions: Vec<Transaction<'_>>,
    ) -> Vec<Result<TransactionResult, SendTransactionError>> {
        let mut results = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let result = tx.send().await;
            results.push(result);
        }
        results
    }

    pub async fn create_block_with_parent(&mut self, parent_hash: &str) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [json!(true), json!(true), json!(parent_hash)],
        });

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        let body = json!({
            "method": "system_health"
        });

        let response = self.call_rpc(body).await?;

        Ok(response.status().is_success())
    }
}
