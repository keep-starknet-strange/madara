use lazy_static::lazy_static;
use mp_starknet::storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_NONCE, STARKNET_STORAGE,
};
use ethers::types::U256;
use sp_io::hashing::twox_128;
use std::collections::HashMap;

pub type StorageWrites<'a> = Vec<(&'a [u8], &'a [u8])>;

lazy_static! {
    static ref SN_NONCE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_NONCE)].concat();
    static ref SN_CONTRACT_CLASS_HASH_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS_HASH)].concat();
    static ref SN_CONTRACT_CLASS_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS)].concat();
    static ref SN_STORAGE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_STORAGE)].concat();
}

/// Data Availablity Mode.
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum DaLayer {
    Celestia,
    Ethereum,
}

impl DaLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            DaLayer::Celestia => "celestia",
            DaLayer::Ethereum => "ethereum",
        }
    }
}

pub enum DaMode {
    Validity,
    Volition,
    Validium,
}

impl DaMode {
    fn as_str(&self) -> &'static str {
        match self {
            DaMode::Validity => "validity",
            DaMode::Volition => "volition",
            DaMode::Validium => "validium",
        }
    }
}

// encode calldata:
// - https://docs.starknet.io/documentation/architecture_and_concepts/Data_Availability/on-chain-data/#pre_v0.11.0_example
pub fn pre_0_11_0_state_diff(storage_diffs: HashMap<&[u8], StorageWrites>, nonces: HashMap<&[u8], &[u8]>) -> Vec<U256> {
    let mut state_diff: Vec<U256> = Vec::new();

    state_diff.push(U256::from(storage_diffs.len()));

    for (addr, writes) in storage_diffs {
        state_diff.push(U256::from_big_endian(addr));
        state_diff.push(U256::from(writes.len()));
        for write in writes {
            state_diff.push(U256::from_big_endian(write.0));
            state_diff.push(U256::from_big_endian(write.1));
        }
    }

    for (addr, nonce) in nonces {
        state_diff.push(U256::from_big_endian(addr));
        state_diff.push(U256::from_big_endian(nonce));
    }
    state_diff
}
