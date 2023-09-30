#![feature(assert_matches)]

use std::cell::Cell;
use std::fmt::Debug;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use anyhow::anyhow;
use constants::ENDING_PORT;
use derive_more::Display;
use lazy_static::lazy_static;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde_json::json;
use starknet_accounts::{
    Account, AccountDeployment, AccountError, AccountFactoryError, Declaration, Execution, LegacyDeclaration,
    OpenZeppelinAccountFactory, SingleOwnerAccount,
};
use starknet_core::types::{DeclareTransactionResult, DeployAccountTransactionResult, InvokeTransactionResult};
use starknet_providers::jsonrpc::{HttpTransport, HttpTransportError, JsonRpcClient, JsonRpcClientError};
use starknet_providers::Provider;
use starknet_signers::local_wallet::SignError;
use starknet_signers::LocalWallet;
use thiserror::Error;
use tokio::sync::Mutex;
use url::Url;

/// Constants (addresses, contracts...)
pub mod constants;
/// Starknet related utilities
pub mod utils;

pub mod fixtures;

type RpcAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>;
pub type RpcOzAccountFactory<'a> = OpenZeppelinAccountFactory<LocalWallet, &'a JsonRpcClient<HttpTransport>>;
type TransactionExecution<'a> = Execution<'a, RpcAccount<'a>>;
type TransactionDeclaration<'a> = Declaration<'a, RpcAccount<'a>>;
type TransactionLegacyDeclaration<'a> = LegacyDeclaration<'a, RpcAccount<'a>>;
type TransactionAccountDeployment<'a> = AccountDeployment<'a, RpcOzAccountFactory<'a>>;
type StarknetAccountError = AccountError<
    <SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> as Account>::SignError,
    <JsonRpcClient<HttpTransport> as Provider>::Error,
>;

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
    AccountFactoryError(AccountFactoryError<SignError, JsonRpcClientError<HttpTransportError>>),
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

lazy_static! {
        /// This is to prevent TOCTOU errors; i.e. one background madara node might find one
        /// port to be free, and while it's trying to start listening to it, another instance
        /// finds that it's free and tries occupying it
        /// Using the mutex in `get_free_port_listener` might be safer than using no mutex at all,
        /// but not sufficiently safe
        static ref FREE_PORT_ATTRIBUTION_MUTEX: Mutex<()> = Mutex::new(());
}

#[derive(Debug)]
/// A wrapper over the Madara process handle, reqwest client and request counter
///
/// When this struct goes out of scope, it's `Drop` impl
/// will take care of killing the Madara process.
pub struct MadaraClient {
    process: Child,
    client: Client,
    rpc_request_count: Cell<usize>,
    starknet_client: JsonRpcClient<HttpTransport>,
    port: u16,
}

#[derive(Display)]
pub enum ExecutionStrategy {
    Native,
    Wasm,
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error("No free ports")]
    NoFreePorts,
}

struct NodePorts {
    rpc_port: u16,
    p2p_port: u16,
}

impl Drop for MadaraClient {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            eprintln!("Could not kill Madara process: {}", e)
        }
    }
}

fn find_available_ports() -> Result<NodePorts, TestError> {
    let mut available_ports = Vec::new();

    for index in 0..3 {
        let mut selected_port = 0;
        let mut port = 1024 + index * 20000 + (std::process::id() % 20000) as u16;

        while selected_port == 0 && port < ENDING_PORT {
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                selected_port = port;
            }
            port += 1;
        }

        if selected_port == 0 {
            return Err(TestError::NoFreePorts);
        }

        available_ports.push(selected_port);
    }

    Ok(NodePorts { rpc_port: available_ports[0], p2p_port: available_ports[1] })
}

impl MadaraClient {
    async fn init(execution: ExecutionStrategy) -> Result<Self, TestError> {
        let NodePorts { p2p_port, rpc_port } = find_available_ports()?;

        let manifest_path = Path::new(&env!("CARGO_MANIFEST_DIR"));
        let repository_root = manifest_path.parent().expect("Failed to get parent directory of CARGO_MANIFEST_DIR");

        std::env::set_current_dir(repository_root).expect("Failed to change working directory");

        let madara_log = std::env::var("MADARA_LOG").unwrap_or_else(|_| "false".to_string());

        Command::new("cargo")
            .stdout(Stdio::null())
            .stderr(if madara_log == "true" { Stdio::inherit() } else { Stdio::null() })
            .args(["run", "--release", "--", "setup"])
            .spawn()
            .expect("Could not setup madara node");

        let child_handle = Command::new("cargo")
		// Silence Madara stdout and stderr
		.stdout(Stdio::null())
		.stderr(if madara_log == "true" { Stdio::inherit() } else { Stdio::null() })
		.args([
			"run",
			"--release",
			"--",
			"run",
			"--sealing=manual",
			&format!("--execution={execution}"),
			"--dev",
            "--tmp",
			&format!("--port={p2p_port}"),
			&format!("--rpc-port={rpc_port}"),
			])
			.spawn()
			.expect("Could not start background madara node");

        let host = &format!("http://localhost:{rpc_port}");

        let starknet_client = JsonRpcClient::new(HttpTransport::new(Url::parse(host).expect("Invalid JSONRPC Url")));

        Ok(MadaraClient {
            process: child_handle,
            client: Client::new(),
            starknet_client,
            rpc_request_count: Default::default(),
            port: rpc_port,
        })
    }

    pub async fn new(execution: ExecutionStrategy) -> Self {
        // we keep the reference, otherwise the mutex unlocks immediately
        let _mutex_guard = FREE_PORT_ATTRIBUTION_MUTEX.lock().await;

        let madara = Self::init(execution).await.expect("Couldn't start Madara Node");

        // Wait until node is ready
        loop {
            match madara.health().await {
                Ok(is_ready) if is_ready => break,
                _ => {}
            }
        }

        madara
    }

    pub async fn run_to_block(&self, target_block: u64) -> anyhow::Result<()> {
        let mut current_block = self.starknet_client.block_number().await?;

        if current_block >= target_block {
            return Err(anyhow!("target_block must be in the future"));
        }

        while current_block < target_block {
            self.create_empty_block().await?;
            current_block += 1;
        }

        Ok(())
    }

    pub async fn create_n_blocks(&self, mut n: u64) -> anyhow::Result<()> {
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

        let response = self
            .client
            .post(&format!("http://localhost:{0}", self.port))
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .body(body)
            .send()
            .await?;

        // Increment rpc_request_count
        let previous = self.rpc_request_count.get();
        self.rpc_request_count.set(previous + 1);

        Ok(response)
    }

    pub fn get_starknet_client(&self) -> &JsonRpcClient<HttpTransport> {
        &self.starknet_client
    }

    pub async fn create_empty_block(&self) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [true, true],
        });

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn create_block_with_txs(
        &self,
        transactions: Vec<Transaction<'_>>,
    ) -> anyhow::Result<Vec<Result<TransactionResult, SendTransactionError>>> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [false, true],
        });

        let mut results = Vec::new();
        for tx in transactions {
            let result = tx.send().await;
            results.push(result);
        }

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(results).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn create_block_with_parent(&self, parent_hash: &str) -> anyhow::Result<()> {
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
