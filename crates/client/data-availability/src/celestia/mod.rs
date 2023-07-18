use std::sync::Arc;

use ethers::types::{Address, I256, U256};

use celestia_rpc::client::{new_http};
use celestia_types::{Blob, nmt::Namespace};
use rand::Rng;


//pub const _STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
//pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";

// TODO:
// - remove unwraps
// - test sequencer address
// - make chain configurable
pub async fn publish_data(eth_node: &str, _sequencer_address: &[u8], state_diff: Vec<U256>) {
    log::info!("publish_data: {:?}", state_diff);
    let client = new_http("https://hooper.au.ngrok.io", Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.qiOWaA7iUn3tuUSn8RklXGpu8Zo6REDErZZhDt75VOU"));

    // cast Vec<U256> to Vec<u8>
    let mut state_diff_bytes: Vec<u8> = Vec::new();
    for i in 0..state_diff.len() {
        let mut bytes = [0 as u8; 32];
        state_diff[i].to_big_endian(&mut bytes);
        state_diff_bytes.extend_from_slice(&bytes);
    }

    //define namespace
    let mut rng = rand::thread_rng();
    let mut array: [u8; 10] = [0; 10];

    for i in 0..10 {
        array[i] = rng.gen();
    }

    let nid = Namespace::new_v0(&array).unwrap();

    //define a new blob
    let blob = Blob::new(nid, state_diff_bytes);
    log::info!("blob: {:?}", blob);

   }

