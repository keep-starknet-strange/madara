use alloc::rc::Rc;
use core::cell::RefCell;
use std::str::FromStr;

use frame_support::bounded_vec;
use sp_core::H256;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::FieldElement;

use crate::crypto::commitment::{
    calculate_declare_tx_hash, calculate_deploy_account_tx_hash, calculate_event_commitment, calculate_event_hash,
    calculate_invoke_tx_hash, calculate_transaction_commitment,
};
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::crypto::hash::{hash, Hasher};
use crate::crypto::merkle_patricia_tree::merkle_node::{BinaryNode, Direction, Node};
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::execution::contract_class_wrapper::ContractClassWrapper;
use crate::execution::types::Felt252Wrapper;
use crate::tests::utils::PEDERSEN_ZERO_HASH;
use crate::traits::hash::{CryptoHasherT, HasherT};
use crate::transaction::types::{
    DeclareTransaction, DeployAccountTransaction, EventWrapper, InvokeTransaction, Transaction, TxType,
};

#[test]
fn test_deploy_account_tx_hash() {
    // Computed with `calculate_deploy_account_transaction_hash` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x050a9c8ed9d8053fc3cf6704b95c1b368cf9a110ff72b87b760db832155b7022").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeployAccountTransaction {
        version: 1,
        calldata: bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE),
        nonce: Felt252Wrapper::ZERO,
        salt: Felt252Wrapper::ZERO,
        signature: bounded_vec!(),
        account_class_hash: Felt252Wrapper::THREE,
        max_fee: Felt252Wrapper::ONE,
    };
    let address = Felt252Wrapper::from(19911991_u64);

    assert_eq!(calculate_deploy_account_tx_hash(transaction, chain_id, address), expected_tx_hash);
}

#[test]
fn test_declare_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x077f205d4855199564663dc9810c1edfcf97573393033dedc3f12dac740aac13").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransaction {
        version: 1,
        sender_address: Felt252Wrapper::from(19911991_u128),
        nonce: Felt252Wrapper::ZERO,
        signature: bounded_vec!(),
        max_fee: Felt252Wrapper::ONE,
        compiled_class_hash: Felt252Wrapper::THREE,
        contract_class: ContractClassWrapper::default(),
    };
    assert_eq!(calculate_declare_tx_hash(transaction, chain_id), expected_tx_hash);
}

#[test]
fn test_invoke_tx_hash() {
    // Computed with `calculate_transaction_hash_common` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x062633b1f3d64708df3d0d44706b388f841ed4534346be6ad60336c8eb2f4b3e").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = InvokeTransaction {
        version: 1,
        sender_address: Felt252Wrapper::from(19911991_u128),
        calldata: bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE),
        nonce: Felt252Wrapper::ZERO,
        signature: bounded_vec!(),
        max_fee: Felt252Wrapper::ONE,
    };
    assert_eq!(calculate_invoke_tx_hash(transaction, chain_id), expected_tx_hash);
}

#[test]
fn test_merkle_tree() {
    let txs = vec![
        Transaction {
            tx_type: TxType::Invoke,
            version: 0_u8,
            hash: Felt252Wrapper::from(6_u128),
            signature: bounded_vec![
                Felt252Wrapper::from(10_u128),
                Felt252Wrapper::from(20_u128),
                Felt252Wrapper::from(30_u128),
            ],
            sender_address: Felt252Wrapper::ZERO,
            nonce: Felt252Wrapper::ZERO,
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: Felt252Wrapper::from(u128::MAX),
        },
        Transaction {
            tx_type: TxType::Invoke,
            version: 0_u8,
            hash: Felt252Wrapper::from(28_u128),
            signature: bounded_vec![Felt252Wrapper::from(40_u128)],
            sender_address: Felt252Wrapper::try_from(&[1; 32]).unwrap(),
            nonce: Felt252Wrapper::ZERO,
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: Felt252Wrapper::from(u128::MAX),
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
    let keys = bounded_vec![Felt252Wrapper::from(2_u128), Felt252Wrapper::from(3_u128),];
    let data = bounded_vec![Felt252Wrapper::from(4_u128), Felt252Wrapper::from(5_u128), Felt252Wrapper::from(6_u128)];
    let from_address = Felt252Wrapper::from(10_u128);
    let transaction_hash = Felt252Wrapper::from(0_u128);
    let event = EventWrapper::new(keys, data, from_address, transaction_hash);
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
    let expected_hash = hash(Hasher::Pedersen(PedersenHasher::default()), &test_data());

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

impl CryptoHasherT for TestCryptoHasher {
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

#[test]
fn test_pedersen_hash_elements_zero() {
    let elements = vec![Felt252Wrapper::ZERO, Felt252Wrapper::ONE];

    let expected_hash = compute_hash_on_elements(&[FieldElement::ZERO, FieldElement::ONE]);
    assert_eq!(PedersenHasher::default().hash_elements(&elements), expected_hash.into());
}

#[test]
fn test_pedersen_hash_elements_empty() {
    let elements = vec![];

    assert_eq!(
        PedersenHasher::default().hash_elements(&elements),
        Felt252Wrapper::from_hex_be(PEDERSEN_ZERO_HASH).unwrap()
    );
}
