use std::sync::Arc;

use ethers::types::{Address, I256, U256};


use celestia_types::{Blob, Commitment};
use celestia_types::nmt::{Namespace, NS_ID_V0_SIZE};
use celestia_rpc::client::{new_http};
use celestia_rpc::HeaderClient;
use celestia_rpc::BlobClient; 

use rand::{Rng, RngCore};


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

pub fn random_ns() -> Namespace {
    Namespace::const_v0(random_bytes_array())
}

pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn random_bytes_array<const N: usize>() -> [u8; N] {
    std::array::from_fn(|_| rand::random())
}

// pub async fn blob_submit<C>(client: &C, blobs: &[Blob]) -> Result<u64, Error>
// where
//     C: ClientT + Sync,
// {
//     let _guard = write_lock().await;
//     client.blob_submit(blobs).await
// }


pub async fn blob_submit_and_get() -> Result<(), Box<dyn std::error::Error>>  {
    let client = new_http("http://localhost:26658",
    Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.qKuJEGn2nai79X3GsGjmpwugDnbdaBzHLK-uXDSn3Bc"))?;

    // client.header_wait_for_height(2).await?;

    let namespace = random_ns();
    let data = random_bytes(5);
    let blob = Blob::new(namespace, data).unwrap();

    let submitted_height = client.blob_submit(&[blob.clone()]).await;

    // let submitted_height = blob_submit(&client, &[blob.clone()]).await.unwrap();
    println!("Submitted height: {:?}", submitted_height); // Print submitted_height

    Ok(())

    // cast Vec<U256> to Vec<u8>
    // let mut state_diff_bytes: Vec<u8> = Vec::new();
    // for i in 0..state_diff.len() {
    //     let mut bytes = [0 as u8; 32];
    //     state_diff[i].to_big_endian(&mut bytes);
    //     state_diff_bytes.extend_from_slice(&bytes);
    // }

    // //define namespace
    // let mut rng = rand::thread_rng();
    // let mut array: [u8; 10] = [0; 10];

    // for i in 0..10 {
    //     array[i] = rng.gen();
    // }

    // let nid = Namespace::new_v0(&array).unwrap();

    // //define a new blob
    // let blob = Blob::new(nid, state_diff_bytes);
    // log::info!("blob: {:?}", blob);

    // let dah = client
    //     .header_get_by_height(submitted_height)
    //     .await
    //     .unwrap()
    //     .dah;

    // let root_hash = dah.row_root(0).unwrap();
    // println!("Root hash: {:?}", root_hash); // Print root_hash

    // let received_blob = client
    //     .blob_get(submitted_height, namespace, blob.commitment)
    //     .await
    //     .unwrap();

    // received_blob.validate().unwrap();
    // assert_eq!(received_blob, blob);

    // let proofs = client
    //     .blob_get_proof(submitted_height, namespace, blob.commitment)
    //     .await
    //     .unwrap();

    // assert_eq!(proofs.len(), 1);

    // let leaves = blob.to_shares().unwrap();

    // proofs[0]
    //     .verify_complete_namespace(&root_hash, &leaves, namespace.into())
    //     .unwrap();
    
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blob_submit_and_get() {
        let result = blob_submit_and_get().await;
        // assert!(result.is_ok());
    }
}