use frame_support::assert_err;
use sp_runtime::offchain::storage::StorageValueRef;

use super::mock::*;
use crate::offchain_worker::{get_eth_rpc_url, OffchainWorkerError};
use crate::ETHEREUM_EXECUTION_RPC;

#[test]
#[ignore]
fn test_get_eth_rpc_url() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let rpc_url = "http://localhost:8545".to_string();
        // test that the rpc url is not set
        assert_err!(get_eth_rpc_url(), OffchainWorkerError::EthRpcNotSet);
        // set the rpc url and test that it is returned correctly
        StorageValueRef::persistent(ETHEREUM_EXECUTION_RPC).set(&"http://localhost:8545".as_bytes().to_vec());
        match get_eth_rpc_url() {
            Ok(url) => assert_eq!(url, rpc_url),
            Err(_) => panic!("Error getting rpc url"),
        }
    })
}
