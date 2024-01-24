use std::sync::Arc;

use anyhow::Error;
use ethers::types::U256;
use mc_commitment_state_diff::BlockDAData;
use mp_felt::Felt252Wrapper;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sp_api::{BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
#[allow(unused_imports)]
use starknet_api::api_core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_core::types::{
    ContractStorageDiffItem, DeclaredClassItem, DeployedContractItem, FieldElement, NonceUpdate, ReplacedClassItem,
    StateDiff, StorageEntry,
};
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

/// Width of the field storing the number of storage updates in `U256`.
#[allow(dead_code)]
const NUM_STORAGE_UPDATES_WIDTH: u64 = 64; // Adjust this based on your logic

/// Macro for converting a value of type `U256` to a StarkNet type.
///
/// Usually in starknet, some types like ClassHash and ContractClass can be directly converted from
/// Felt252Wrapper, so we use this macro to convert U256 into Felt252Wrapper and then into the
/// corresponding data.
///
/// # Arguments
///
/// * `$data` - The value to be converted.
/// * `$target_type` - The StarkNet type to convert to.
///
/// # Returns
///
/// A `Result` containing the converted value or an `Error` if the conversion fails.
///
/// # Example
///
/// ```rust
/// // Example usage
/// let result = convert_to_starknet_type!(U256::from(1), ClassHash);
/// ```
macro_rules! convert_to_starknet_type {
    ($data:expr, $target_type:ident) => {{
        let result = Felt252Wrapper::try_from($data).map(|ft| $target_type::from(ft));
        result
    }};
}

/// Checks if a contract is deployed based on its address and block hash.
///
/// # Arguments
///
/// * `address` - Address of the contract.
/// * `block_hash` - Hash of the block in which the contract is being checked.
/// * `client` - StarkNet runtime client.
///
/// # Returns
///
/// A `Result` indicating whether the contract is deployed or not.
///
/// # Example
///
/// ```rust
/// // Example usage
/// let result = contract_deployed(address, block_hash, client);
/// ```
#[allow(unused)]
fn contract_deployed<B, C>(address: ContractAddress, block_hash: B::Hash, client: Arc<C>) -> Result<bool, Error>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    #[cfg(test)]
    // When testing, we have only an empty client.
    return Ok(false);

    #[cfg(not(test))]
    match client.runtime_api().contract_class_hash_by_address(block_hash, address) {
        Ok(class_hash) => Ok(!class_hash.eq(&ClassHash::default())),
        Err(e) => Ok(false),
    }
}

/// Decodes a state difference using the starknet v0.11.0 logic.
///
/// # Arguments
///
/// * `encoded_diff` - Encoded state difference data in the form of `U256` slices.
/// * `block_hash` - Hash of the block in which the state difference occurred.
/// * `client` - StarkNet runtime client.
///
/// # Returns
///
/// A `Result` containing the decoded `StateDiff` or an `Error` if decoding fails.
///
/// # Example
///
/// ```rust
/// // Example usage
/// let result = decode_011_diff(&encoded_diff, block_hash, client);
/// ```
pub fn decode_011_diff<B, C>(encoded_diff: &[U256], block_hash: B::Hash, client: Arc<C>) -> Result<StateDiff, Error>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    // Offset is set to 5 as the 5 first items are part of the DA header.
    let mut offset = 5;
    let num_contract_updates = encoded_diff[offset].low_u64();
    offset += 1;

    let mut nonces = Vec::new();
    let mut deployed_contracts = Vec::new();
    let mut declared_classes = Vec::new();
    let mut replaced_classes = Vec::new();
    let mut storage_diffs = Vec::new();
    let deprecated_declared_classes = Vec::new();

    for _ in 0..num_contract_updates {
        let address = convert_to_starknet_type!(encoded_diff[offset], ContractAddress)?;
        offset += 1;

        let summary = encoded_diff[offset];
        offset += 1;

        let num_storage_updates = summary.low_u64();
        let class_info_flag = summary.bit(128);
        let nonce_value = summary >> 64;

        nonces.push(NonceUpdate {
            contract_address: address.0.0.into(),
            nonce: convert_to_starknet_type!(nonce_value, FieldElement)?,
        });

        if class_info_flag {
            let class_hash = convert_to_starknet_type!(encoded_diff[offset], FieldElement)?;
            offset += 1;
            if contract_deployed(address, block_hash, client.clone())? {
                replaced_classes.push(ReplacedClassItem { contract_address: address.0.0.into(), class_hash });
            } else {
                deployed_contracts.push(DeployedContractItem { address: address.0.0.into(), class_hash });
            }
        }

        if num_storage_updates > 0 {
            let mut storage_entries = Vec::new();
            for _ in 0..num_storage_updates {
                let key = convert_to_starknet_type!(encoded_diff[offset], FieldElement)?;
                offset += 1;

                storage_entries
                    .push(StorageEntry { key, value: convert_to_starknet_type!(encoded_diff[offset], FieldElement)? });
                offset += 1;
            }
            storage_diffs.push(ContractStorageDiffItem { address: address.0.0.into(), storage_entries });
        }
    }

    let num_declared_classes = encoded_diff[offset].low_u64();
    offset += 1;
    for _ in 0..num_declared_classes {
        let class_hash = convert_to_starknet_type!(encoded_diff[offset], FieldElement)?;
        offset += 1;

        let compiled_class_hash = convert_to_starknet_type!(encoded_diff[offset], FieldElement)?;
        declared_classes.push(DeclaredClassItem { class_hash, compiled_class_hash });
        offset += 1;
    }

    Ok(StateDiff {
        deployed_contracts,
        storage_diffs,
        declared_classes,
        deprecated_declared_classes,
        nonces,
        replaced_classes,
    })
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
