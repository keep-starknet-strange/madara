use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature = "avail")]
use mc_data_availability::avail::{config::AvailConfig, AvailClient};
#[cfg(feature = "celestia")]
use mc_data_availability::celestia::{config::CelestiaConfig, CelestiaClient};
use mc_data_availability::ethereum::config::EthereumConfig;
use mc_data_availability::ethereum::EthereumClient;
use mc_data_availability::{DaClient, DaLayer};
use serde::de::DeserializeOwned;

#[cfg(feature = "avail")]
use crate::constants::AVAIL_DA_CONFIG;
#[cfg(feature = "celestia")]
use crate::constants::CELESTIA_DA_CONFIG;
use crate::constants::ETHEREUM_DA_CONFIG;

fn load_da_config<C: DeserializeOwned>(path: &Path) -> C {
    let file = File::open(path).expect("path shoud lead to an existing file");
    serde_json::from_reader(file).expect("path content should be a valid DA config")
}

pub fn get_da_client(da_layer: DaLayer) -> Box<dyn DaClient + Send + Sync> {
    let da_path = get_da_path(da_layer);

    let da_client: Box<dyn DaClient + Send + Sync> = match da_layer {
        #[cfg(feature = "celestia")]
        DaLayer::Celestia => {
            let celestia_conf = load_da_config::<CelestiaConfig>(&da_path);
            Box::new(CelestiaClient::try_from(celestia_conf).expect("Failed to create Celestia client"))
        }
        DaLayer::Ethereum => {
            let ethereum_conf = load_da_config::<EthereumConfig>(&da_path);
            Box::new(EthereumClient::try_from(ethereum_conf).expect("Failed to create Ethereum client"))
        }
        #[cfg(feature = "avail")]
        DaLayer::Avail => {
            let avail_conf = load_da_config::<AvailConfig>(&da_path);
            Box::new(AvailClient::try_from(avail_conf).expect("Failed to create Avail client"))
        }
    };

    da_client
}

pub(crate) fn get_da_path(da_layer: DaLayer) -> PathBuf {
    match da_layer {
        #[cfg(feature = "celestia")]
        DaLayer::Celestia => CELESTIA_DA_CONFIG.into(),
        DaLayer::Ethereum => ETHEREUM_DA_CONFIG.into(),
        #[cfg(feature = "avail")]
        DaLayer::Avail => AVAIL_DA_CONFIG.into(),
    }
}
