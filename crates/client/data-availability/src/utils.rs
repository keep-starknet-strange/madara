use ethers::types::U256;
use starknet_api::api_core::{Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::ThinStateDiff;
use url::{ParseError, Url};

const CLASS_FLAG_TRUE: &str = "0x100000000000000000000000000000001"; // 2 ^ 128 + 1
const NONCE_BASE: &str = "0x10000000000000000"; // 2 ^ 64

/// Encode DA Calldata:
/// - https://docs.starknet.io/documentation/architecture_and_concepts/Data_Availability/on-chain-data
pub fn state_diff_to_calldata(mut state_diff: ThinStateDiff, num_addrs_accessed: usize) -> Vec<U256> {
    let mut calldata: Vec<U256> = Vec::new();

    calldata.push(U256::from(num_addrs_accessed));

    for (addr, writes) in state_diff.storage_diffs {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        let class_flag = state_diff.deployed_contracts.remove(&addr);
        let nonce = state_diff.nonces.remove(&addr);
        calldata.push(da_word(class_flag.is_some(), nonce, writes.len() as u64));

        if class_flag.is_some() {
            calldata.push(U256::from_big_endian(class_flag.unwrap().0.bytes()));
        }

        for (key, val) in &writes {
            calldata.push(U256::from_big_endian(key.0.key().bytes()));
            calldata.push(U256::from_big_endian(val.bytes()));
        }
    }
    for (addr, nonce) in state_diff.nonces {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        let class_flag = state_diff.deployed_contracts.remove(&addr);
        calldata.push(da_word(class_flag.is_some(), Some(nonce), 0_u64));
        if class_flag.is_some() {
            calldata.push(U256::from_big_endian(class_flag.unwrap().0.bytes()));
        }
    }
    for (addr, class_hash) in state_diff.deployed_contracts {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        calldata.push(da_word(true, None, 0_u64));
        calldata.push(U256::from_big_endian(class_hash.0.bytes()));
    }

    calldata.push(U256::from(state_diff.declared_classes.len()));

    for (class_hash, compiled_class_hash) in &state_diff.declared_classes {
        calldata.push(U256::from_big_endian(class_hash.0.bytes()));
        calldata.push(U256::from_big_endian(compiled_class_hash.0.bytes()));
    }

    calldata
}

///  |---padding---|---class flag---|---new nonce---|---num changes---|
///      127 bits        1 bit           64 bits         64 bits
pub fn da_word(class_flag: bool, nonce_change: Option<Nonce>, num_changes: u64) -> U256 {
    let mut word = U256::from(0);

    if class_flag {
        word += U256::from_str_radix(CLASS_FLAG_TRUE, 16).unwrap();
    }
    if let Some(new_nonce) = nonce_change {
        word += U256::from_big_endian(&new_nonce.0.bytes()) + U256::from_str_radix(NONCE_BASE, 16).unwrap();
    }
    word += U256::from(num_changes);

    println!("DA WORD: {} {nonce_change:?} {num_changes} {word}", u64::MAX);

    word
}

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

pub fn safe_split(key: &[u8]) -> ([u8; 16], [u8; 16], Option<Vec<u8>>) {
    let length = key.len();
    let (mut prefix, mut child, mut rest) = ([0_u8; 16], [0_u8; 16], None);
    if length <= 16 {
        prefix[..length].copy_from_slice(&key[..])
    }
    if length > 16 && key.len() <= 32 {
        prefix.copy_from_slice(&key[..16]);
        child[..(length - 16)].copy_from_slice(&key[16..]);
    }
    if length > 32 {
        prefix.copy_from_slice(&key[..16]);
        child.copy_from_slice(&key[16..32]);
        rest = Some(Vec::from(&key[32..]))
    }

    (prefix, child, rest)
}

pub fn bytes_to_felt(raw: &[u8]) -> StarkFelt {
    let mut buf = [0_u8; 32];
    if raw.len() < 32 {
        buf[32 - raw.len()..].copy_from_slice(raw);
    } else {
        buf.copy_from_slice(&raw[..32]);
    }
    StarkFelt::new(buf).unwrap()
}

pub fn bytes_to_key(raw: &[u8]) -> PatriciaKey {
    PatriciaKey(bytes_to_felt(raw))
}
