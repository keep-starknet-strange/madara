use std::cell::Cell;
use std::fmt::Debug;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use anyhow::anyhow;
use derive_more::Display;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
/// A wrapper over the Madara process handle, reqwest client and request counter
///
/// When this struct goes out of scope, it's `Drop` impl
/// will take care of killing the Madara process.
pub struct MadaraClient {
    process: Child,
    client: Client,
    rpc_request_count: Cell<usize>,
}

#[derive(Display)]
pub enum ExecutionStrategy {
    Native,
    Wasm,
}

impl Drop for MadaraClient {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            eprintln!("Could not kill Madara process: {}", e)
        }
    }
}

impl MadaraClient {
    fn init(execution: ExecutionStrategy) -> Self {
        let madara_path = Path::new("../target/release/madara");
        assert!(
            madara_path.exists(),
            "could not find the madara binary at `{}`",
            madara_path.to_str().expect("madara_path must be a valid path")
        );

        let child_handle = Command::new(madara_path.to_str().unwrap())
                // Silence Madara stdout and stderr          
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .args([
                    "--alice",
                    "--sealing=manual",
                    &format!("--execution={execution}"),
                    "--chain=dev",
                    "--tmp"
                ])
                .spawn()
                .unwrap();

        MadaraClient { process: child_handle, client: Client::new(), rpc_request_count: Default::default() }
    }

    pub async fn new(execution: ExecutionStrategy) -> Self {
        let madara = Self::init(execution);

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
        let mut current_block = self.get_block_number().await?;

        if current_block >= target_block {
            return Err(anyhow!("target_block must be in the future"));
        }

        while current_block < target_block {
            self.create_block().await?;
            current_block += 1;
        }

        Ok(())
    }

    pub async fn create_n_blocks(&self, mut n: u64) -> anyhow::Result<()> {
        while n > 0 {
            self.create_block().await?;
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
            .post("http://localhost:9944")
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .body(body)
            .send()
            .await?;

        // Increment rpc_request_count
        let previous = self.rpc_request_count.get();
        self.rpc_request_count.set(previous + 1);

        Ok(response)
    }
}

// Substrate RPC
impl MadaraClient {
    pub async fn create_block(&self) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": vec![true, true],
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

#[derive(Debug, Deserialize)]
struct MadaraRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: JsonRpcVersion,
    #[allow(dead_code)]
    id: usize,
    result: T,
}

#[derive(Debug)]
enum JsonRpcVersion {
    V2,
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;

        if s == "2.0" {
            Ok(JsonRpcVersion::V2)
        } else {
            Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &"a valid version. `2.0` is supported"))
        }
    }
}

type GetBlockNumberResponse = MadaraRpcResponse<u64>;

// Starknet RPC
impl MadaraClient {
    pub async fn get_block_number(&self) -> anyhow::Result<u64> {
        let body = json!({
            "method": "starknet_blockNumber"
        });

        let response = self.call_rpc(body).await?;
        let bytes = response.bytes().await?;

        let body: GetBlockNumberResponse = serde_json::from_slice(&bytes)?;

        Ok(body.result)
    }
}
