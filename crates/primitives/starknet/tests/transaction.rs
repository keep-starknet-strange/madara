use frame_support::bounded_vec;
use mp_starknet::transaction::types::{EventWrapper, Transaction, TxType};
use sp_core::{H256, U256};

#[test]
fn verify_tx_version_passes_for_valid_version() {
    let tx = Transaction {
        version: 1_u8,
        hash: H256::from_low_u64_be(6),
        signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        events: bounded_vec![EventWrapper::default(), EventWrapper::default()],
        sender_address: [0; 32],
        nonce: U256::zero(),
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::InvokeTx).is_ok())
}

#[test]
fn verify_tx_version_fails_for_invalid_version() {
    let tx = Transaction {
        version: 0_u8,
        hash: H256::from_low_u64_be(6),
        signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        events: bounded_vec![EventWrapper::default(), EventWrapper::default()],
        sender_address: [0; 32],
        nonce: U256::zero(),
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::InvokeTx).is_err())
}
