use std::str::FromStr;

use frame_support::bounded_vec;
use kp_starknet::crypto::commitment::{
    calculate_event_commitment, calculate_event_hash, calculate_transaction_commitment,
};
use kp_starknet::crypto::hash::pedersen::PedersenHasher;
use kp_starknet::transaction::{Event, Transaction};
use sp_core::{H256, U256};
use starknet_crypto::FieldElement;

#[test]
fn test_merkle_tree() {
    let txs = vec![
        Transaction {
            version: U256::zero(),
            hash: H256::from_low_u64_be(6),
            signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
            events: bounded_vec![Event::default(), Event::default()],
        },
        Transaction {
            version: U256::zero(),
            hash: H256::from_low_u64_be(28),
            signature: bounded_vec![H256::from_low_u64_be(40)],
            events: bounded_vec![],
        },
    ];
    let tx_com = calculate_transaction_commitment::<PedersenHasher>(&txs);
    let event_com = calculate_event_commitment::<PedersenHasher>(&txs);
    // The values we test ours against are computed from the sequencer test.
    assert_eq!(
        H256::from_str("0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02").unwrap(),
        event_com.0
    );
    assert_eq!(H256::from_str("0x054c0fddf3aaf1ca03271712b323822647b66042ccc418ba1d7fb852aebfd2da").unwrap(), tx_com);
}

#[test]
fn test_event_hash() {
    let keys = bounded_vec![H256::from_low_u64_be(2), H256::from_low_u64_be(3),];
    let data = bounded_vec![H256::from_low_u64_be(4), H256::from_low_u64_be(5), H256::from_low_u64_be(6)];
    let from_address = H256::from_low_u64_be(10);
    let event = Event::new(keys, data, from_address);
    assert_eq!(
        calculate_event_hash::<PedersenHasher>(&event),
        FieldElement::from_str("0x3f44fb0516121d225664058ecc7e415c4725d6a7a11fd7d515c55c34ef8270b").unwrap()
    );

    assert_eq!(
        calculate_event_hash::<PedersenHasher>(&Event { from_address, ..Event::empty() }),
        FieldElement::from_str("0x754233cddfc3670a8e9c47f714397312a0319691a8762a49351fad896b37462").unwrap()
    )
}
