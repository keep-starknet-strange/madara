use blockifier::execution::contract_class::ContractClass;
use madara_runtime::{AuraConfig, EnableManualSeal, GrandpaConfig, RuntimeGenesisConfig, SystemConfig, WASM_BINARY};
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use pallet_starknet::types::ContractStorageKeyWrapper;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{Pair, Public, H256};
use sp_state_machine::BasicExternalities;
use starknet_core::types::FieldElement;
use starknet_core::utils::get_storage_var_address;

use super::constants::*;

pub const ACCOUNT_PUBLIC_KEY: &str = "0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

/// Starknet testnet SN_GOERLI
pub const CHAIN_ID_STARKNET_TESTNET: u128 = 0x534e5f474f45524c49;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
    /// Genesis config.
    genesis_config: RuntimeGenesisConfig,
    /// The flag that if enable manual-seal mode.
    enable_manual_seal: Option<bool>,
}

/// If enable_manual_seal is true, then the runtime storage variable EnableManualSeal will be set to
/// true. This is just a common way to pass information from the chain spec to the runtime.
impl sp_runtime::BuildStorage for DevGenesisExt {
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        BasicExternalities::execute_with_storage(storage, || {
            if let Some(enable_manual_seal) = &self.enable_manual_seal {
                EnableManualSeal::set(enable_manual_seal);
            }
        });
        self.genesis_config.assimilate_storage(storage)
    }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None).expect("static values are valid; qed").public()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config(enable_manual_seal: Option<bool>) -> Result<DevChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(DevChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            DevGenesisExt {
                genesis_config: testnet_genesis(
                    wasm_binary,
                    // Initial PoA authorities
                    vec![authority_keys_from_seed("Alice")],
                    true,
                ),
                enable_manual_seal,
            }
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                    authority_keys_from_seed("Charlie"),
                    authority_keys_from_seed("Dave"),
                    authority_keys_from_seed("Eve"),
                    authority_keys_from_seed("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        None,
        // Extensions
        None,
    ))
}

pub fn get_contract_class(contract_content: &'static [u8]) -> ContractClass {
    serde_json::from_slice(contract_content).unwrap()
}

/// Returns the storage key for a given storage name, keys and offset.
/// Calculates pedersen(sn_keccak(storage_name), keys) + storage_key_offset which is the key in the
/// starknet contract for storage_name(key_1, key_2, ..., key_n).
/// https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/#storage_variables
pub fn get_storage_key(
    address: &Felt252Wrapper,
    storage_name: &str,
    keys: &[Felt252Wrapper],
    storage_key_offset: u64,
) -> ContractStorageKeyWrapper {
    let storage_key_offset = H256::from_low_u64_be(storage_key_offset);
    let mut storage_key = get_storage_var_address(
        storage_name,
        keys.iter().map(|x| FieldElement::from(*x)).collect::<Vec<_>>().as_slice(),
    )
    .unwrap();
    storage_key += FieldElement::from_bytes_be(&storage_key_offset.to_fixed_bytes()).unwrap();
    (*address, storage_key.into())
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> RuntimeGenesisConfig {
    // ACCOUNT CONTRACT
    let no_validate_account_class =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/NoValidateAccount.json")).try_into().unwrap();
    let no_validate_account_class_hash = Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_CLASS_HASH).unwrap();
    let no_validate_account_address = Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_ADDRESS).unwrap();

    // ARGENT ACCOUNT CONTRACT
    let argent_account_class =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/ArgentAccount.json")).try_into().unwrap();
    let argent_account_class_hash = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH).unwrap();
    let argent_account_address = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_ADDRESS).unwrap();
    let argent_proxy_class =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/Proxy.json")).try_into().unwrap();
    let argent_proxy_class_hash = Felt252Wrapper::from_hex_be(ARGENT_PROXY_CLASS_HASH).unwrap();

    // TEST CONTRACT
    let test_contract_class =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/test.json")).try_into().unwrap();
    let test_contract_class_hash = Felt252Wrapper::from_hex_be(TEST_CONTRACT_CLASS_HASH).unwrap();
    let test_contract_address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();

    // Fee token
    let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let fee_token_class_hash = Felt252Wrapper::from_hex_be(FEE_TOKEN_CLASS_HASH).unwrap();

    // ERC20 CONTRACT
    let erc20_class: ContractClassWrapper =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/ERC20.json")).try_into().unwrap();
    let token_class_hash = Felt252Wrapper::from_hex_be(ERC20_CLASS_HASH).unwrap();
    let token_contract_address = Felt252Wrapper::from_hex_be(ERC20_ADDRESS).unwrap();

    // ERC721 CONTRACT
    let erc721_class: ContractClassWrapper =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/ERC721.json")).try_into().unwrap();
    let nft_class_hash = Felt252Wrapper::from_hex_be(ERC721_CLASS_HASH).unwrap();
    let nft_contract_address = Felt252Wrapper::from_hex_be(ERC721_ADDRESS).unwrap();

    // UDC CONTRACT
    let udc_class: ContractClassWrapper =
        get_contract_class(include_bytes!("../../../cairo-contracts/build/UniversalDeployer.json")).try_into().unwrap();
    let udc_class_hash = Felt252Wrapper::from_hex_be(UDC_CLASS_HASH).unwrap();
    let udc_contract_address = Felt252Wrapper::from_hex_be(UDC_CONTRACT_ADDRESS).unwrap();

    let public_key = Felt252Wrapper::from_hex_be(PUBLIC_KEY).unwrap();
    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(&CHAIN_ID_STARKNET_TESTNET.to_be_bytes()).unwrap());

    RuntimeGenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        // Authority-based consensus protocol used for block production
        aura: AuraConfig { authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect() },
        // Deterministic finality mechanism used for block finalization
        grandpa: GrandpaConfig { authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect() },
        /// Starknet Genesis configuration.
        starknet: madara_runtime::pallet_starknet::GenesisConfig {
            contracts: vec![
                (no_validate_account_address, no_validate_account_class_hash),
                (test_contract_address, test_contract_class_hash),
                (token_contract_address, token_class_hash),
                (token_contract_address, token_class_hash),
                (nft_contract_address, nft_class_hash),
                (fee_token_address, fee_token_class_hash),
                (argent_account_address, argent_account_class_hash),
                (udc_contract_address, udc_class_hash),
            ],
            contract_classes: vec![
                (no_validate_account_class_hash, no_validate_account_class),
                (argent_account_class_hash, argent_account_class),
                (argent_proxy_class_hash, argent_proxy_class),
                (test_contract_class_hash, test_contract_class),
                (token_class_hash, erc20_class.clone()),
                (fee_token_class_hash, erc20_class),
                (nft_class_hash, erc721_class),
                (udc_class_hash, udc_class),
            ],
            storage: vec![
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[no_validate_account_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[no_validate_account_address], 1),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[argent_account_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&token_contract_address, "ERC20_balances", &[no_validate_account_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&token_contract_address, "ERC20_balances", &[no_validate_account_address], 1),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[public_key], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&argent_account_address, "_signer", &[], 0),
                    Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
                ),
                (
                    get_storage_key(&nft_contract_address, "Ownable_owner", &[], 0),
                    Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_ADDRESS).unwrap(),
                ),
            ],
            fee_token_address,
            _phantom: Default::default(),
            chain_id,
        },
    }
}
