use ethers::types::U256;
use mc_commitment_state_diff::BlockDAData;
use starknet_api::api_core::{Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use url::{ParseError, Url};

const CLASS_FLAG_TRUE: &str = "0x100000000000000000000000000000001"; // 2 ^ 128 + 1
const NONCE_BASE: &str = "0xFFFFFFFFFFFFFFFF"; // 2 ^ 64 - 1

/// DA calldata encoding:
/// - https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/on-chain-data
pub fn block_data_to_calldata(mut block_da_data: BlockDAData) -> Vec<U256> {
    // pushing the headers and num_addr_accessed
    let mut calldata: Vec<U256> = vec![
        U256::from_big_endian(&block_da_data.previous_state_root.0), // prev merkle root
        U256::from_big_endian(&block_da_data.new_state_root.0),      // new merkle root
        U256::from(block_da_data.block_number),                      // block number
        U256::from_big_endian(&block_da_data.block_hash.0.0),        // block hash
        U256::from_big_endian(&block_da_data.config_hash.0),         // config hash,
        U256::from(block_da_data.num_addr_accessed),                 // num_addr_accessed
    ];

    // Loop over storage diffs
    for (addr, writes) in block_da_data.state_diff.storage_diffs {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        let class_flag = block_da_data
            .state_diff
            .deployed_contracts
            .get(&addr)
            .or_else(|| block_da_data.state_diff.replaced_classes.get(&addr));

        let nonce = block_da_data.state_diff.nonces.remove(&addr);
        calldata.push(da_word(class_flag.is_some(), nonce, writes.len() as u64));

        if let Some(class_hash) = class_flag {
            calldata.push(U256::from_big_endian(class_hash.0.bytes()));
        }

        for (key, val) in &writes {
            calldata.push(U256::from_big_endian(key.0.key().bytes()));
            calldata.push(U256::from_big_endian(val.bytes()));
        }
    }

    // Handle nonces
    for (addr, nonce) in block_da_data.state_diff.nonces {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        let class_flag = block_da_data
            .state_diff
            .deployed_contracts
            .get(&addr)
            .or_else(|| block_da_data.state_diff.replaced_classes.get(&addr));

        calldata.push(da_word(class_flag.is_some(), Some(nonce), 0_u64));
        if let Some(class_hash) = class_flag {
            calldata.push(U256::from_big_endian(class_hash.0.bytes()));
        }
    }

    // Handle deployed contracts
    for (addr, class_hash) in block_da_data.state_diff.deployed_contracts {
        calldata.push(U256::from_big_endian(&addr.0.key().0));

        calldata.push(da_word(true, None, 0_u64));
        calldata.push(U256::from_big_endian(class_hash.0.bytes()));
    }

    // Handle declared classes
    calldata.push(U256::from(block_da_data.state_diff.declared_classes.len()));

    for (class_hash, compiled_class_hash) in &block_da_data.state_diff.declared_classes {
        calldata.push(U256::from_big_endian(class_hash.0.bytes()));
        calldata.push(U256::from_big_endian(compiled_class_hash.0.bytes()));
    }

    calldata
}

/// DA word encoding:
/// |---padding---|---class flag---|---new nonce---|---num changes---|
///     127 bits        1 bit           64 bits          64 bits
pub fn da_word(class_flag: bool, nonce_change: Option<Nonce>, num_changes: u64) -> U256 {
    let mut word = U256::from(0);

    if class_flag {
        word += U256::from_str_radix(CLASS_FLAG_TRUE, 16).unwrap();
    }
    if let Some(new_nonce) = nonce_change {
        word += U256::from_big_endian(new_nonce.0.bytes()) + U256::from_str_radix(NONCE_BASE, 16).unwrap();
    }

    word += U256::from(num_changes);

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

pub fn safe_split(key: &[u8]) -> ([u8; 16], Option<Vec<u8>>) {
    let length = key.len();
    let (mut child, mut rest) = ([0_u8; 16], None);
    if length > 16 && key.len() <= 32 {
        child[..(length - 16)].copy_from_slice(&key[16..]);
    } else if length > 32 {
        child.copy_from_slice(&key[16..32]);
        rest = Some(Vec::from(&key[32..]))
    }

    (child, rest)
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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use starknet_api::stark_felt;

    use super::*;

    #[rstest]
    #[case(false, 1, 1, "18446744073709551617")]
    #[case(false, 1, 0, "18446744073709551616")]
    #[case(false, 0, 6, "6")]
    #[case(true, 1, 0, "340282366920938463481821351505477763073")]
    fn da_word_works(
        #[case] class_flag: bool,
        #[case] new_nonce: u64,
        #[case] num_changes: u64,
        #[case] expected: String,
    ) {
        let new_nonce = if new_nonce > 0 { Some(Nonce(stark_felt!(new_nonce))) } else { None };
        let da_word = da_word(class_flag, new_nonce, num_changes);
        let expected = U256::from_str_radix(&expected, 10).unwrap();
        assert_eq!(da_word, expected);
    }
}
