mod constants;

use std::path::PathBuf;

use constants::{GENESIS_ASSETS_DIR, GENESIS_ASSETS_FILE};
use jsonrpsee::types::error::CallError;
use mp_genesis_config::GenesisData;

pub trait GenesisProvider {
    type LoadGenesisDataError: std::error::Error;
    fn load_genesis_data(&self) -> Result<GenesisData, LoadGenesisDataError>;
}

pub struct OnDiskGenesisConfig(pub PathBuf);

#[derive(thiserror::Error, Debug)]
pub enum LoadGenesisDataError {
    #[error("File cannot be deserialized into a GenesisData struct")]
    InvalidJson,
    #[error("Unable to read genesis file: invalid path")]
    InvalidPath,
}

impl From<LoadGenesisDataError> for jsonrpsee::core::Error {
    fn from(e: LoadGenesisDataError) -> Self {
        jsonrpsee::core::Error::Call(CallError::Failed(anyhow::Error::from(e)))
    }
}

impl GenesisProvider for OnDiskGenesisConfig {
    type LoadGenesisDataError = LoadGenesisDataError;

    fn load_genesis_data(&self) -> Result<GenesisData, Self::LoadGenesisDataError> {
        let genesis_path = self.0.join(GENESIS_ASSETS_DIR).join(GENESIS_ASSETS_FILE);

        log::info!("Loading genesis data at: {}", genesis_path.display());

        std::fs::read_to_string(genesis_path.clone()).map_or(Err(LoadGenesisDataError::InvalidPath), |s| {
            serde_json::from_str::<GenesisData>(&s).map_err(|_| LoadGenesisDataError::InvalidJson)
        })
    }
}
