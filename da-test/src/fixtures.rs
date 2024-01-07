use clap::ValueEnum;
use mc_data_availability::{DaClient, DaLayer};
use rstest::fixture;

use crate::utils::get_da_client;

#[fixture]
pub fn da_client() -> Box<dyn DaClient + Send + Sync> {
    let da_layer_str = std::env::var("DA_LAYER").expect("DA_LAYER env var not set");
    let da_layer = DaLayer::from_str(&da_layer_str, true).expect("Invalid value for DA_LAYER");

    get_da_client(da_layer)
}
