use std::path::PathBuf;

use mc_data_availability::avail::config::AvailConfig;
use mc_data_availability::avail::AvailClient;
use mc_data_availability::celestia::config::CelestiaConfig;
use mc_data_availability::celestia::CelestiaClient;
use mc_data_availability::ethereum::config::EthereumConfig;
use mc_data_availability::ethereum::EthereumClient;
use mc_data_availability::{DaClient, DaLayer};

use crate::constants::{AVAIL_DA_CONFIG, CELESTIA_DA_CONFIG, ETHEREUM_DA_CONFIG};

pub fn get_da_client(da_layer: DaLayer) -> Box<dyn DaClient + Send + Sync> {
    let da_path = get_da_path(da_layer);

    let da_client: Box<dyn DaClient + Send + Sync> = match da_layer {
        DaLayer::Celestia => {
            let celestia_conf = CelestiaConfig::try_from(&da_path).expect("Failed to read Celestia config");
            Box::new(CelestiaClient::try_from(celestia_conf).expect("Failed to create Celestia client"))
        }
        DaLayer::Ethereum => {
            let ethereum_conf = EthereumConfig::try_from(&da_path).expect("Failed to read Ethereum config");
            Box::new(EthereumClient::try_from(ethereum_conf).expect("Failed to create Ethereum client"))
        }
        DaLayer::Avail => {
            let avail_conf = AvailConfig::try_from(&da_path).expect("Failed to read Avail config");
            Box::new(AvailClient::try_from(avail_conf).expect("Failed to create Avail client"))
        }
    };

    da_client
}

pub(crate) fn get_da_path(da_layer: DaLayer) -> PathBuf {
    match da_layer {
        DaLayer::Celestia => CELESTIA_DA_CONFIG.clone(),
        DaLayer::Ethereum => ETHEREUM_DA_CONFIG.clone(),
        DaLayer::Avail => AVAIL_DA_CONFIG.clone(),
    }
}