use std::sync::Arc;

use ethers::types::{Address, I256, U256};

use celestia_rpc::client::{new_http};

//pub const _STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
//pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";

// TODO:
// - remove unwraps
// - test sequencer address
// - make chain configurable
pub async fn publish_data(eth_node: &str, _sequencer_address: &[u8], state_diff: Vec<U256>) {
    log::info!("publish_data: {:?}", state_diff);

    let client = new_http("https://hooper.au.ngrok.io", Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.qiOWaA7iUn3tuUSn8RklXGpu8Zo6REDErZZhDt75VOU"));
}

