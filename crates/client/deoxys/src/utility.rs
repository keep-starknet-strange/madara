//! Utility functions.

/// Returns the block number of the last block synced by the node.
pub async fn get_last_synced_block(rpc_port: u16) -> u64 {
    let client = reqwest::Client::new();

    let url = format!("http://localhost:{}/", rpc_port);
    let payload = serde_json::to_vec(&serde_json::json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "chain_getBlock",
        "params": []
    }))
    .unwrap();

    let response: serde_json::Value = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .body(payload)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let number_as_hex = response["result"]["block"]["header"]["number"].as_str().unwrap();
    u64::from_str_radix(&number_as_hex[2..], 16).unwrap()
}
