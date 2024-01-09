use std::path::PathBuf;

#[cfg(feature = "avail")]
use mc_data_availability::avail::{config::AvailConfig, AvailClient};
#[cfg(feature = "celestia")]
use mc_data_availability::celestia::{config::CelestiaConfig, CelestiaClient};
use mc_data_availability::ethereum::config::EthereumConfig;
use mc_data_availability::ethereum::EthereumClient;
use mc_data_availability::{DaClient, DaLayer};

#[cfg(feature = "avail")]
use crate::constants::AVAIL_DA_CONFIG;
#[cfg(feature = "celestia")]
use crate::constants::CELESTIA_DA_CONFIG;
use crate::constants::ETHEREUM_DA_CONFIG;

pub fn get_da_client(da_layer: DaLayer) -> Box<dyn DaClient + Send + Sync> {
    let da_path = get_da_path(da_layer);

    let da_client: Box<dyn DaClient + Send + Sync> = match da_layer {
        #[cfg(feature = "celestia")]
        DaLayer::Celestia => {
            let celestia_conf = CelestiaConfig::try_from(&da_path).expect("Failed to read Celestia config");
            Box::new(CelestiaClient::try_from(celestia_conf).expect("Failed to create Celestia client"))
        }
        DaLayer::Ethereum => {
            let ethereum_conf = EthereumConfig::try_from(&da_path).expect("Failed to read Ethereum config");
            Box::new(EthereumClient::try_from(ethereum_conf).expect("Failed to create Ethereum client"))
        }
        #[cfg(feature = "avail")]
        DaLayer::Avail => {
            let avail_conf = AvailConfig::try_from(&da_path).expect("Failed to read Avail config");
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
