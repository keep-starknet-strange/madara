use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::deprecated_contract_class::ContractClass as DeprecatedContractClass;
use starknet_api::hash::StarkFelt;
use starknet_api::state::{ContractClass, StorageKey};

use super::*;

#[allow(dead_code)]
const NUM_STORAGE_UPDATES_WIDTH: u64 = 64; // Adjust this based on your logic

/// Usually in starknet, some types like ClassHash and ContractClass can be directly converted from
/// Felt252Wrapper, so we use this macro to convert U256 into Felt252Wrapper and then into the
/// corresponding data.
///
/// Usage: convert_to_starknet_type!(U256::from(1), ClassHash)
macro_rules! convert_to_starknet_type {
    ($data:expr, $target_type:ident) => {{
        let result = Felt252Wrapper::try_from($data)
            .map(|ft| $target_type::from(ft))
            .map_err(|e| Error::TypeError(e.to_string()));
        result
    }};
}

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
        Err(e) => Err(Error::Other(e.to_string())),
    }
}

pub fn decode_011_diff<B, C>(encoded_diff: &[U256], block_hash: B::Hash, client: Arc<C>) -> Result<StateDiff, Error>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    let mut offset = 0;
    let num_contract_updates = encoded_diff[offset].low_u64();
    offset += 1;

    let mut nonces: IndexMap<ContractAddress, Nonce> = IndexMap::new();
    let mut deployed_contracts: IndexMap<ContractAddress, ClassHash> = IndexMap::new();
    let mut declared_classes: IndexMap<ClassHash, (CompiledClassHash, ContractClass)> = IndexMap::new();
    let mut replaced_classes: IndexMap<ContractAddress, ClassHash> = IndexMap::new();
    let mut storage_diffs: IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>> = IndexMap::new();
    let deprecated_declared_classes: IndexMap<ClassHash, DeprecatedContractClass> = IndexMap::new();

    for _ in 0..num_contract_updates {
        let address = convert_to_starknet_type!(encoded_diff[offset], ContractAddress)?;
        offset += 1;

        let summary = encoded_diff[offset];
        offset += 1;

        let num_storage_updates = summary.low_u64();
        let class_info_flag = summary.bit(128);
        let nonce_value = summary >> 64;

        nonces.insert(address, convert_to_starknet_type!(nonce_value, Nonce)?);

        if class_info_flag {
            let class_hash = convert_to_starknet_type!(encoded_diff[offset], ClassHash)?;
            offset += 1;
            if contract_deployed(address, block_hash, client.clone())? {
                replaced_classes.insert(address, class_hash);
            } else {
                deployed_contracts.insert(address, class_hash);
            }
        }

        if num_storage_updates > 0 {
            let mut diffs = IndexMap::new();
            for _ in 0..num_storage_updates {
                let key = convert_to_starknet_type!(encoded_diff[offset], StorageKey)?;
                offset += 1;

                diffs.insert(key, convert_to_starknet_type!(encoded_diff[offset], StarkFelt)?);
                offset += 1;
            }
            storage_diffs.insert(address, diffs);
        }
    }

    let num_declared_classes = encoded_diff[offset].low_u64();
    offset += 1;
    for _ in 0..num_declared_classes {
        let class_hash = convert_to_starknet_type!(encoded_diff[offset], ClassHash)?;
        offset += 1;

        let compiled_class_hash = convert_to_starknet_type!(encoded_diff[offset], CompiledClassHash)?;
        declared_classes.insert(class_hash, (compiled_class_hash, ContractClass::default()));
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

#[allow(dead_code)]
pub fn decode_pre_011_diff(encoded_diff: &[U256], with_constructor_args: bool) -> Result<StateDiff, Error> {
    let mut offset = 0;
    let num_deployments_cells = encoded_diff[offset].as_usize();
    offset += 1;

    let mut deployed_contracts: IndexMap<ContractAddress, ClassHash> = IndexMap::new();
    let mut nonces: IndexMap<ContractAddress, Nonce> = IndexMap::new();
    let declared_classes: IndexMap<ClassHash, (CompiledClassHash, ContractClass)> = IndexMap::new();
    let replaced_classes: IndexMap<ContractAddress, ClassHash> = IndexMap::new();
    let mut storage_diffs: IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>> = IndexMap::new();
    let deprecated_declared_classes: IndexMap<ClassHash, DeprecatedContractClass> = IndexMap::new();

    // Parse contract deployments
    while offset - 1 < num_deployments_cells {
        let address = convert_to_starknet_type!(encoded_diff[offset], ContractAddress)?;

        offset += 1;
        let class_hash = convert_to_starknet_type!(encoded_diff[offset], ClassHash)?;

        offset += 1;
        deployed_contracts.insert(address, class_hash);

        if with_constructor_args {
            let constructor_args_len = encoded_diff[offset].as_usize();
            offset += 1;
            offset += constructor_args_len; //as usize;
        }
    }

    let updates_len = encoded_diff[offset].as_usize();
    offset += 1;
    for _i in 0..updates_len {
        let address = convert_to_starknet_type!(encoded_diff[offset], ContractAddress)?;
        offset += 1;

        let num_updates = encoded_diff[offset].low_u64();

        match Felt252Wrapper::try_from(encoded_diff[offset] >> NUM_STORAGE_UPDATES_WIDTH) {
            Ok(contract_address) => {
                nonces.insert(address, Nonce::from(contract_address));
            }
            Err(err) => {
                return Err(Error::TypeError(err.to_string()));
            }
        }
        offset += 1;

        let mut diffs = IndexMap::new();
        for _ in 0..num_updates {
            let key = convert_to_starknet_type!(encoded_diff[offset], StorageKey)?;

            offset += 1;

            let value = convert_to_starknet_type!(encoded_diff[offset], StarkFelt)?;
            offset += 1;
            diffs.insert(key, value);
        }
        storage_diffs.insert(address, diffs);
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
