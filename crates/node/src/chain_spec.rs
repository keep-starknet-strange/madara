use blockifier::execution::contract_class::ContractClass;
use madara_runtime::{
    AccountId, AuraConfig, BalancesConfig, EnableManualSeal, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use pallet_starknet::types::ContractStorageKeyWrapper;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{sr25519, Pair, Public, H256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
use starknet_core::types::FieldElement;
use starknet_core::utils::get_storage_var_address;

use super::constants::*;

pub const ACCOUNT_PUBLIC_KEY: &str = "0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2";

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

/// Starknet testnet SN_GOERLI
pub const CHAIN_ID_STARKNET_TESTNET: u128 = 0x534e5f474f45524c49;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
    /// Genesis config.
    genesis_config: GenesisConfig,
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

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
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
                    // Sudo account
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    // Pre-funded accounts
                    vec![
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_account_id_from_seed::<sr25519::Public>("Charlie"),
                        get_account_id_from_seed::<sr25519::Public>("Dave"),
                        get_account_id_from_seed::<sr25519::Public>("Eve"),
                        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                    ],
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
                vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
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
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    let account_class =
        get_contract_class(include_bytes!("../../../resources/account/simple/account.json")).try_into().unwrap();
    let argent_account_class =
        get_contract_class(include_bytes!("../../../resources/account/argent/account.json")).try_into().unwrap();
    let argent_proxy_class =
        get_contract_class(include_bytes!("../../../resources/account/argent/proxy/proxy.json")).try_into().unwrap();
    let test_class = get_contract_class(include_bytes!("../../../resources/test.json")).try_into().unwrap();
    let erc20_class: ContractClassWrapper =
        get_contract_class(include_bytes!("../../../resources/erc20/erc20.json")).try_into().unwrap();

    // ACCOUNT CONTRACT
    let contract_address = Felt252Wrapper::from_hex_be(CONTRACT_ADDRESS).unwrap();

    let class_hash = Felt252Wrapper::from_hex_be(CLASS_HASH).unwrap();

    // ARGENT ACCOUNT CONTRACT
    let argent_account_address = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_ADDRESS).unwrap();

    let argent_account_class_hash = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH).unwrap();

    let argent_proxy_class_hash = Felt252Wrapper::from_hex_be(ARGENT_PROXY_CLASS_HASH).unwrap();

    // TEST CONTRACT
    let other_contract_address = Felt252Wrapper::from_hex_be(OTHER_CONTRACT_ADDRESS).unwrap();

    let other_class_hash = Felt252Wrapper::from_hex_be(OTHER_CLASS_HASH).unwrap();

    // Fee token
    let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    let fee_token_class_hash = Felt252Wrapper::from_hex_be(FEE_TOKEN_CLASS_HASH).unwrap();

    // ERC20 CONTRACT
    let token_contract_address = Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_ADDRESS).unwrap();

    let token_class_hash = Felt252Wrapper::from_hex_be(TOKEN_CLASS_HASH).unwrap();

    let public_key = Felt252Wrapper::from_hex_be(PUBLIC_KEY).unwrap();

    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        // Provides interaction with balances and accounts
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
        },
        // Authority-based consensus protocol used for block production
        aura: AuraConfig { authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect() },
        // Deterministic finality mechanism used for block finalization
        grandpa: GrandpaConfig { authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect() },
        // Allows executing privileged functions
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        // Provides the logic needed to handle transaction fees
        transaction_payment: Default::default(),
        /// Starknet Genesis configuration.
        starknet: madara_runtime::pallet_starknet::GenesisConfig {
            contracts: vec![
                (contract_address, class_hash),
                (other_contract_address, other_class_hash),
                (token_contract_address, token_class_hash),
                (token_contract_address, token_class_hash),
                (fee_token_address, fee_token_class_hash),
                (argent_account_address, argent_account_class_hash),
            ],
            contract_classes: vec![
                (class_hash, account_class),
                (argent_account_class_hash, argent_account_class),
                (argent_proxy_class_hash, argent_proxy_class),
                (other_class_hash, test_class),
                (token_class_hash, erc20_class.clone()),
                (fee_token_class_hash, erc20_class),
            ],
            storage: vec![
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[contract_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[contract_address], 1),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&fee_token_address, "ERC20_balances", &[argent_account_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&token_contract_address, "ERC20_balances", &[contract_address], 0),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    get_storage_key(&token_contract_address, "ERC20_balances", &[contract_address], 1),
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
            ],
            fee_token_address,
            _phantom: Default::default(),
            chain_id: CHAIN_ID_STARKNET_TESTNET,
        },
    }
}
