use std::str::FromStr;

use frame_support::bounded_vec;
use kp_starknet::crypto::commitment::calculate_transaction_commitment;
use kp_starknet::transaction::Transaction;
use sp_core::{H256, U256};

#[test]
fn test_merkle_tree() {
    let txs = vec![
        Transaction {
            version: U256::zero(),
            hash: H256::from_low_u64_be(6),
            signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        },
        Transaction {
            version: U256::zero(),
            hash: H256::from_low_u64_be(28),
            signature: bounded_vec![H256::from_low_u64_be(40)],
        },
    ];
    let com = calculate_transaction_commitment(&txs);
    assert_eq!(H256::from_str("0x054c0fddf3aaf1ca03271712b323822647b66042ccc418ba1d7fb852aebfd2da").unwrap(), com)
}
