use std::str::FromStr;

use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH};
use hex::FromHex;
use madara_runtime::{
    AccountId, AuraConfig, BalancesConfig, EnableManualSeal, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use mp_starknet::execution::ContractClassWrapper;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{sr25519, Pair, Public, H256, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

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

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    let account_class = get_contract_class(ACCOUNT_CONTRACT_PATH);

    let test_class = get_contract_class(include_bytes!("../../../resources/test.json"));
    let erc20_class = get_contract_class(include_bytes!("../../../resources/erc20/erc20.json"));

    // ACCOUNT CONTRACT
    let contract_address_bytes =
        <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let class_hash_bytes =
        <[u8; 32]>::from_hex("025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918").unwrap();

    // TEST CONTRACT
    let other_contract_address_bytes =
        <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000001111").unwrap();
    let other_class_hash_bytes =
        <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000001000").unwrap();

    // Fee token
    let fee_token_address =
        <[u8; 32]>::from_hex("040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00").unwrap();

    // ERC20 CONTRACT
    let token_contract_address_str = "040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00";
    let token_contract_address_bytes = <[u8; 32]>::from_hex(token_contract_address_str).unwrap();

    let token_class_hash_str = "0000000000000000000000000000000000000000000000000000000000010000";
    let token_class_hash_bytes = <[u8; 32]>::from_hex(token_class_hash_str).unwrap();

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
                (contract_address_bytes, class_hash_bytes),
                (other_contract_address_bytes, other_class_hash_bytes),
                (token_contract_address_bytes, token_class_hash_bytes),
            ],
            contract_classes: vec![
                (class_hash_bytes, ContractClassWrapper::from(account_class)),
                (other_class_hash_bytes, ContractClassWrapper::from(test_class)),
                (token_class_hash_bytes, ContractClassWrapper::from(erc20_class)),
            ],
            storage: vec![
                (
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) which is the key in the starknet contract for
                        // ERC20_balances(0x01).low
                        H256::from_str("0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09").unwrap(),
                    ),
                    U256::from(u128::MAX),
                ),
                (
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) + 1 which is the key in the starknet contract
                        // for ERC20_balances(0x01).high
                        H256::from_str("0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f0A").unwrap(),
                    ),
                    U256::from(u128::MAX),
                ),
            ],
            fee_token_address,
            _phantom: Default::default(),
        },
    }
}
