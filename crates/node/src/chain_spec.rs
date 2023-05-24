use blockifier::execution::contract_class::ContractClass;
use madara_runtime::{
    AccountId, AuraConfig, BalancesConfig, EnableManualSeal, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;

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
    let contract_address = Felt252Wrapper::from_hex_be("0x1").unwrap();

    let class_hash =
        Felt252Wrapper::from_hex_be("0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f").unwrap();

    // ARGENT ACCOUNT CONTRACT
    let argent_account_address = Felt252Wrapper::from_hex_be("0x2").unwrap();

    let argent_account_class_hash =
        Felt252Wrapper::from_hex_be("0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d").unwrap();

    let argent_proxy_class_hash =
        Felt252Wrapper::from_hex_be("0x0424b7f61e3c5dfd74400d96fdea7e1f0bf2757f31df04387eaa957f095dd7b9").unwrap();

    // TEST CONTRACT
    let other_contract_address = Felt252Wrapper::from_hex_be("0x1111").unwrap();

    let other_class_hash = Felt252Wrapper::from_hex_be("0x1000").unwrap();

    // Fee token
    let fee_token_address =
        Felt252Wrapper::from_hex_be("0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d01").unwrap();

    let fee_token_class_hash = Felt252Wrapper::from_hex_be("0x20000").unwrap();

    // ERC20 CONTRACT
    let token_contract_address =
        Felt252Wrapper::from_hex_be("0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00").unwrap();

    let token_class_hash = Felt252Wrapper::from_hex_be("0x10000").unwrap();

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
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) which is the key in the starknet contract for
                        // ERC20_balances(0x01).low
                        Felt252Wrapper::from_hex_be(
                            "0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) + 1 which is the key in the starknet contract
                        // for ERC20_balances(0x01).high
                        Felt252Wrapper::from_hex_be(
                            "0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f0A",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x02) which is the key in the starknet contract
                        // for ERC20_balances(0x02).low
                        Felt252Wrapper::from_hex_be(
                            "0x01d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        token_contract_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) which is the key in the starknet contract for
                        // ERC20_balances(0x01).low
                        Felt252Wrapper::from_hex_be(
                            "0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        token_contract_address,
                        // pedersen(sn_keccak(b"ERC20_balances"), 0x01) + 1 which is the key in the starknet contract
                        // for ERC20_balances(0x01).high
                        Felt252Wrapper::from_hex_be(
                            "0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f0A",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        fee_token_address,
                        // pedersen(sn_keccak(b"ERC20_balances"),
                        // 0x03b8268ca24c43fa43cf8200ec43bd7c508a92bc318c25a83bc031b48233804d) which is the key in the
                        // starknet contract for
                        // ERC20_balances(0x03b8268ca24c43fa43cf8200ec43bd7c508a92bc318c25a83bc031b48233804d).low
                        Felt252Wrapper::from_hex_be(
                            "0x067fdeb147e1d955ee5049d653043a991c811ed3de90746bb2d4b48a5f229d52",
                        )
                        .unwrap(),
                    ),
                    Felt252Wrapper::from(u128::MAX),
                ),
                (
                    (
                        argent_account_address.into(),
                        // pedersen(sn_keccak(b"_signer"))
                        H256::from_str("0x01ccc09c8a19948e048de7add6929589945e25f22059c7345aaf7837188d8d05")
                            .unwrap()
                            .into(),
                    ),
                    Felt252Wrapper(U256::from_str_radix(ACCOUNT_PUBLIC_KEY, 16).unwrap()),
                ),
            ],
            fee_token_address,
            _phantom: Default::default(),
            chain_id: CHAIN_ID_STARKNET_TESTNET,
        },
    }
}
