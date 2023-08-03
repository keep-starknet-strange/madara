use lazy_static::lazy_static;
use madara_runtime::{
    AuraConfig, EnableManualSeal, GenesisConfig, GrandpaConfig, Runtime, StarknetConfig, SystemConfig, WASM_BINARY,
};
use pallet_starknet::genesis_loader::{read_file_to_string, GenesisLoader};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{Pair, Public};
use sp_state_machine::BasicExternalities;

lazy_static! {
    static ref STARKNET_GENESIS: String = read_file_to_string("crates/node/src/genesis_assets/genesis.json");
}

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
            // Logging the development account
            print_development_accounts();

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

// helper to print development accounts info
// accounts with addresses 0x1 and 0x4 are NO VALIDATE accounts (don't require PK)
// accounts with addresses 0x2 and 0x3 have the same PK
pub fn print_development_accounts() {
    let loader: GenesisLoader = serde_json::from_str(&*STARKNET_GENESIS).unwrap();
    let starknet_genesis: madara_runtime::pallet_starknet::GenesisConfig<Runtime> = loader.into();
    let no_validate_account_address = starknet_genesis.contracts[0].0.0.to_string();
    let argent_account_address = starknet_genesis.contracts[1].0.0.to_string();
    let oz_account_address = starknet_genesis.contracts[2].0.0.to_string();
    let cairo_1_no_validate_account_address = starknet_genesis.contracts[3].0.0.to_string();

    const ARGENT_PK: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
    log::info!("🧪 Using the following development accounts:");
    log::info!("🧪 NO VALIDATE with address: {} and pk: {}", no_validate_account_address, "");
    log::info!("🧪 ARGENT with address: {} and pk: {}", argent_account_address, ARGENT_PK);
    log::info!("🧪 OZ with address: {} and pk: {}", oz_account_address, ARGENT_PK);
    log::info!("🧪 CAIRO 1 with address: {} and pk: {}", cairo_1_no_validate_account_address, "");
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
                // Intended to be only 2
                vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
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
    _enable_println: bool,
) -> GenesisConfig {
    let loader: GenesisLoader = serde_json::from_str(&*STARKNET_GENESIS).unwrap();
    let starknet_genesis: madara_runtime::pallet_starknet::GenesisConfig<Runtime> = loader.into();
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        // Authority-based consensus protocol used for block production
        aura: AuraConfig { authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect() },
        // Deterministic finality mechanism used for block finalization
        grandpa: GrandpaConfig { authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect() },
        /// Starknet Genesis configuration.
        starknet: starknet_genesis,
    }
}
