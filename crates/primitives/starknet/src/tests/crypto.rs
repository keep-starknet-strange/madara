use alloc::rc::Rc;
use core::cell::RefCell;
use std::str::FromStr;

use frame_support::bounded_vec;
use sp_core::{H256, U256};
use starknet_crypto::FieldElement;

use crate::crypto::commitment::{calculate_event_commitment, calculate_event_hash, calculate_transaction_commitment};
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::crypto::hash::{hash, HashType};
use crate::crypto::merkle_patricia_tree::merkle_node::{BinaryNode, Direction, Node};
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::traits::hash::{CryptoHasher, Hasher};
use crate::transaction::types::{EventWrapper, Transaction};

#[test]
fn test_merkle_tree() {
    let txs = vec![
        Transaction {
            version: 0_u8,
            hash: H256::from_low_u64_be(6),
            signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
            sender_address: [0; 32],
            nonce: U256::zero(),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: U256::MAX,
        },
        Transaction {
            version: 0_u8,
            hash: H256::from_low_u64_be(28),
            signature: bounded_vec![H256::from_low_u64_be(40)],
            sender_address: [1; 32],
            nonce: U256::zero(),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: U256::MAX,
        },
    ];
    let tx_com = calculate_transaction_commitment::<PedersenHasher>(&txs);
    let events = vec![EventWrapper::default(), EventWrapper::default()];
    let event_com = calculate_event_commitment::<PedersenHasher>(&events);
    // The values we test ours against are computed from the sequencer test.
    assert_eq!(
        H256::from_str("0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02").unwrap(),
        event_com
    );
    assert_eq!(H256::from_str("0x054c0fddf3aaf1ca03271712b323822647b66042ccc418ba1d7fb852aebfd2da").unwrap(), tx_com);
}

#[test]
fn test_event_hash() {
    let keys = bounded_vec![H256::from_low_u64_be(2), H256::from_low_u64_be(3),];
    let data = bounded_vec![H256::from_low_u64_be(4), H256::from_low_u64_be(5), H256::from_low_u64_be(6)];
    let from_address = H256::from_low_u64_be(10).to_fixed_bytes();
    let event = EventWrapper::new(keys, data, from_address);
    assert_eq!(
        calculate_event_hash::<PedersenHasher>(&event),
        FieldElement::from_str("0x3f44fb0516121d225664058ecc7e415c4725d6a7a11fd7d515c55c34ef8270b").unwrap()
    );

    assert_eq!(
        calculate_event_hash::<PedersenHasher>(&EventWrapper { from_address, ..EventWrapper::empty() }),
        FieldElement::from_str("0x754233cddfc3670a8e9c47f714397312a0319691a8762a49351fad896b37462").unwrap()
    )
}

#[test]
fn test_pedersen_hash() {
    let pedersen_hasher = PedersenHasher::default();
    let hash_result = pedersen_hasher.hash(&test_data());
    let expected_hash = hash(HashType::Pedersen, &test_data());

    assert_eq!(hash_result, expected_hash);
}

// test_data() function returns a Vec<u8> as an example data
fn test_data() -> Vec<u8> {
    vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
        31, 32,
    ]
}

struct TestCryptoHasher;

impl CryptoHasher for TestCryptoHasher {
    fn hash(a: FieldElement, b: FieldElement) -> FieldElement {
        a + b
    }

    fn compute_hash_on_elements(elements: &[FieldElement]) -> FieldElement {
        if elements.is_empty() {
            FieldElement::ZERO
        } else {
            let hash = elements.iter().fold(FieldElement::ZERO, |a, b| a + *b);
            hash
        }
    }
}

#[test]
fn test_binary_node_functions() {
    let binary_node = BinaryNode {
        hash: Some(FieldElement::from(1_u32)),
        height: 0,
        left: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(2_u32)))),
        right: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(3_u32)))),
    };

    let unresolved_node = Node::Unresolved(FieldElement::from(6_u32));

    assert_eq!(binary_node.get_child(Direction::Left).borrow().hash(), Some(FieldElement::from(2_u32)));
    assert_eq!(binary_node.get_child(Direction::Right).borrow().hash(), Some(FieldElement::from(3_u32)));

    assert_eq!(binary_node.hash, Some(FieldElement::from(1_u32)));

    assert!(!unresolved_node.is_empty());
    assert!(!unresolved_node.is_binary());
}

#[test]
fn test_direction_invert() {
    let left = Direction::Left;
    let right = Direction::Right;

    assert_eq!(left.invert(), Direction::Right);
    assert_eq!(right.invert(), Direction::Left);
}

#[test]
fn test_binary_node_calculate_hash() {
    let mut binary_node = BinaryNode {
        hash: None,
        height: 0,
        left: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(2_u32)))),
        right: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(3_u32)))),
    };

    binary_node.calculate_hash::<TestCryptoHasher>();
    assert_eq!(binary_node.hash, Some(FieldElement::from(5_u32)));
}

#[test]
fn test_binary_node_implementations() {
    let test_node = BinaryNode {
        hash: None,
        height: 0,
        left: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(2_u32)))),
        right: Rc::new(RefCell::new(Node::Leaf(FieldElement::from(3_u32)))),
    };

    // Test Display trait implementation
    let node_string = format!("{:?}", test_node);
    assert_eq!(
        node_string,
        "BinaryNode { hash: None, height: 0, left: RefCell { value: Leaf(FieldElement { inner: \
         0x0000000000000000000000000000000000000000000000000000000000000002 }) }, right: RefCell { value: \
         Leaf(FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000003 }) } }"
    );

    // Test Debug trait implementation
    let debug_string = format!("{:?}", test_node);
    assert_eq!(
        debug_string,
        "BinaryNode { hash: None, height: 0, left: RefCell { value: Leaf(FieldElement { inner: \
         0x0000000000000000000000000000000000000000000000000000000000000002 }) }, right: RefCell { value: \
         Leaf(FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000003 }) } }"
    );
}
