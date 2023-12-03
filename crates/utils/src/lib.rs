mod constants;

use std::path::Path;

use constants::{GENESIS_ASSETS_DIR, GENESIS_ASSETS_FILE};
use pallet_starknet::genesis_loader::GenesisData;

pub trait GenesisProvider {
    fn load_genesis_data(&self) -> Result<GenesisData, LoadGenesisDataError>;
}

pub struct OnDiskGenesisConfig(pub Box<Path>);

#[derive(Debug)]
pub struct LoadGenesisDataError<'a>(&'a str);

impl GenesisProvider for OnDiskGenesisConfig {
    fn load_genesis_data(&self) -> Result<GenesisData, LoadGenesisDataError> {
        let genesis_path = self.0.join(GENESIS_ASSETS_DIR).join(GENESIS_ASSETS_FILE);

        log::info!("Loading predeployed accounts at: {}", genesis_path.display());

        let genesis_file_content = std::fs::read_to_string(genesis_path.clone()).unwrap_or_else(|_| {
            panic!(
                "Failed to read genesis file at {}. Please run `madara setup` before opening an issue.",
                genesis_path.canonicalize().unwrap().display()
            )
        });
        let genesis_data: GenesisData = serde_json::from_str(&genesis_file_content).expect("Failed loading genesis");

        Ok(genesis_data)
    }
}
