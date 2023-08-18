use ethers::types::U256;
use url::{ParseError, Url};

pub fn get_bytes_from_state_diff(state_diff: &[U256]) -> Vec<u8> {
    let state_diff_bytes: Vec<u8> = state_diff
        .iter()
        .flat_map(|item| {
            let mut bytes = [0_u8; 32];
            item.to_big_endian(&mut bytes);
            bytes.to_vec()
        })
        .collect();

    state_diff_bytes
}

pub fn get_valid_url(endpoint: &str) -> Result<Url, ParseError> {
    Url::parse(endpoint)
}

pub fn is_valid_ws_endpoint(endpoint: &str) -> bool {
    if let Ok(url) = get_valid_url(endpoint) { matches!(url.scheme(), "ws" | "wss") } else { false }
}

pub fn is_valid_http_endpoint(endpoint: &str) -> bool {
    if let Ok(url) = get_valid_url(endpoint) { matches!(url.scheme(), "http" | "https") } else { false }
}
