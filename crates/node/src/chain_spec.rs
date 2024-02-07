use std::path::PathBuf;

use madara_runtime::{AuraConfig, GrandpaConfig, RuntimeGenesisConfig, SealingMode, SystemConfig, WASM_BINARY};
use mp_felt::Felt252Wrapper;
use pallet_starknet::genesis_loader::{GenesisData, GenesisLoader, HexFelt};
use sc_service::{BasePath, ChainType};
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use sp_core::{Pair, Public};
use sp_state_machine::BasicExternalities;

use crate::constants::DEV_CHAIN_ID;

pub const GENESIS_ASSETS_DIR: &str = "genesis-assets/";
pub const GENESIS_ASSETS_FILE: &str = "genesis.json";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
    /// Genesis config.
    genesis_config: RuntimeGenesisConfig,
    /// The sealing mode being used.
    sealing: SealingMode,
}

/// The `sealing` from the `DevGenesisExt` is passed to the runtime via the storage. The runtime
/// can then use this information to adjust accordingly. This is just a common way to pass
/// information from the chain spec to the runtime.
///
/// NOTE: if `sealing` is `None`, then the runtime will use the default sealing mode.
impl sp_runtime::BuildStorage for DevGenesisExt {
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        BasicExternalities::execute_with_storage(storage, || {
            madara_runtime::Sealing::set(&self.sealing);
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

pub fn development_config(sealing: SealingMode, base_path: BasePath) -> Result<DevChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let chain_id = DEV_CHAIN_ID;

    Ok(DevChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        chain_id,
        ChainType::Development,
        move || {
            let genesis_loader = load_genesis(base_path.config_dir(chain_id));

            // Logging the development account
            print_development_accounts(&genesis_loader);

            DevGenesisExt {
                genesis_config: testnet_genesis(
                    genesis_loader,
                    wasm_binary,
                    // Initial PoA authorities
                    vec![authority_keys_from_seed("Alice")],
                    true,
                ),
                sealing: sealing.clone(),
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
pub fn print_development_accounts(genesis_loader: &GenesisLoader) {
    // TODO: this is only true by luck. It's not enforced by anything
    let no_validate_account_address = genesis_loader.data().contracts[0].0.0;
    let argent_account_address = genesis_loader.data().contracts[1].0.0;
    let oz_account_address = genesis_loader.data().contracts[2].0.0;
    let cairo_1_no_validate_account_address = genesis_loader.data().contracts[3].0.0;

    let argent_pk: HexFelt =
        Felt252Wrapper::from_hex_be("0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d")
            .unwrap()
            .into();
    log::info!("🧪 Using the following development accounts:");
    log::info!("🧪 NO VALIDATE with address: {no_validate_account_address:#x} and no pk");
    log::info!("🧪 ARGENT with address: {argent_account_address:#x} and pk: {argent_pk:#x}");
    log::info!("🧪 OZ with address: {oz_account_address:#x} and pk: {argent_pk:#x}");
    log::info!("🧪 CAIRO 1 with address: {cairo_1_no_validate_account_address:#x} and no pk");
}

pub fn local_testnet_config(base_path: BasePath, chain_id: &str) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let owned_chain_id = chain_id.to_owned();

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        chain_id,
        ChainType::Local,
        move || {
            testnet_genesis(
                load_genesis(base_path.config_dir(&owned_chain_id)),
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

fn load_genesis(data_path: PathBuf) -> GenesisLoader {
    let genesis_path = data_path.join(GENESIS_ASSETS_DIR).join(GENESIS_ASSETS_FILE);
    log::debug!("🧪 Loading genesis data at : {}", genesis_path.display());
    let genesis_file_content = std::fs::read_to_string(genesis_path)
        .expect("Failed to read genesis file. Please run `madara setup` before opening an issue.");
    let genesis_data: GenesisData = serde_json::from_str(&genesis_file_content).expect("Failed loading genesis");

    GenesisLoader::new(data_path, genesis_data)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    genesis_loader: GenesisLoader,
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> RuntimeGenesisConfig {
    let starknet_genesis_config: madara_runtime::pallet_starknet::GenesisConfig<_> = genesis_loader.into();

    RuntimeGenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            _config: Default::default(),
        },
        // Authority-based consensus protocol used for block production
        aura: AuraConfig { authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect() },
        // Deterministic finality mechanism used for block finalization
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
            _config: Default::default(),
        },
        /// Starknet Genesis configuration.
        starknet: starknet_genesis_config,
        rancici: Default::default(),
    }
}
