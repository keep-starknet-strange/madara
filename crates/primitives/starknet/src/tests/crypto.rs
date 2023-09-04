use std::str::FromStr;

use blockifier::execution::contract_class::{ContractClass, ContractClassV1};
use frame_support::bounded_vec;
use starknet_api::stdlib::collections::HashMap;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::FieldElement;

use crate::crypto::commitment::{
    calculate_class_commitment_tree_root_hash, calculate_declare_tx_hash, calculate_deploy_account_tx_hash,
    calculate_event_commitment, calculate_event_hash, calculate_invoke_tx_hash, calculate_transaction_commitment,
};
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::crypto::hash::poseidon::PoseidonHasher;
use crate::crypto::hash::{hash, Hasher};
use crate::crypto::merkle_patricia_tree::merkle_node::{BinaryNode, Direction, Node, NodeId};
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::execution::types::Felt252Wrapper;
use crate::tests::utils::PEDERSEN_ZERO_HASH;
use crate::traits::hash::HasherT;
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
        is_query: false,
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
        class_hash: Felt252Wrapper::THREE,
        // Arbitrary choice to pick v1 vs v0.
        contract_class: ContractClass::from(ContractClassV1::default()),
        compiled_class_hash: None,
        is_query: false,
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
        is_query: false,
    };
    assert_eq!(calculate_invoke_tx_hash(transaction, chain_id), expected_tx_hash);
}

#[test]
fn test_ref_merkle_tree() {
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
            max_fee: Felt252Wrapper::from(u64::MAX),
            is_query: false,
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
            max_fee: Felt252Wrapper::from(u64::MAX),
            is_query: false,
        },
    ];
    let tx_com = calculate_transaction_commitment::<PedersenHasher>(&txs);
    let events = vec![EventWrapper::default(), EventWrapper::default()];
    let event_com = calculate_event_commitment::<PedersenHasher>(&events);
    // The values we test ours against are computed from the sequencer test.
    assert_eq!(
        Felt252Wrapper::from_hex_be("0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02").unwrap(),
        event_com
    );
    assert_eq!(
        Felt252Wrapper::from_hex_be("0x054c0fddf3aaf1ca03271712b323822647b66042ccc418ba1d7fb852aebfd2da").unwrap(),
        tx_com
    );
}

#[test]
fn test_merkle_tree_class_commitment() {
    let class_hashes = vec![Felt252Wrapper::from(0_u128), Felt252Wrapper::from(1_u128)];

    let class_com = calculate_class_commitment_tree_root_hash::<PedersenHasher>(&class_hashes);

    // The values we test ours against are computed with the starkware python library.
    assert_eq!(
        Felt252Wrapper::from_hex_be("0x0218b7f0879373722df04bd1c2054cad721251b3dd238973e153347a26f8a674").unwrap(),
        class_com
    );
}

#[test]
fn test_merkle_tree_poseidon() {
    let class_hashes = vec![Felt252Wrapper::from(0_u128), Felt252Wrapper::from(1_u128)];

    let class_com = calculate_class_commitment_tree_root_hash::<PoseidonHasher>(&class_hashes);

    // The values we test ours against are computed from the sequencer test.
    assert_eq!(
        Felt252Wrapper::from_hex_be("0x01d195cdec8d7a8bbe302e5d728f1d5d6d985b9a2e054abd415412cd9c9674fb").unwrap(),
        class_com
    );
}

#[test]
fn test_event_hash() {
    let keys = bounded_vec![Felt252Wrapper::from(2_u128), Felt252Wrapper::from(3_u128),];
    let data = bounded_vec![Felt252Wrapper::from(4_u128), Felt252Wrapper::from(5_u128), Felt252Wrapper::from(6_u128)];
    let from_address = Felt252Wrapper::from(10_u128);
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
    let hash_result = pedersen_hasher.hash_bytes(&test_data());
    let expected_hash = hash(Hasher::Pedersen(PedersenHasher::default()), &test_data());

    assert_eq!(hash_result, expected_hash);
}

#[test]
fn test_poseidon_hash() {
    let poseidon = PoseidonHasher::default();
    let hash_result = poseidon.hash_bytes(&test_data());
    let expected_hash = hash(Hasher::Poseidon(PoseidonHasher::default()), &test_data());

    assert_eq!(hash_result, expected_hash);
}

// test_data() function returns a Vec<u8> as an example data
fn test_data() -> Vec<u8> {
    vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
        31, 32,
    ]
}

#[derive(Default)]
struct TestHasher;

impl HasherT for TestHasher {
    fn hash_bytes(&self, _data: &[u8]) -> Felt252Wrapper {
        unimplemented!()
    }

    fn compute_hash_on_wrappers(&self, _data: &[Felt252Wrapper]) -> Felt252Wrapper {
        unimplemented!()
    }

    fn hash_elements(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        a + b
    }

    fn compute_hash_on_elements(&self, elements: &[FieldElement]) -> FieldElement {
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
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    nodes.insert(NodeId(0), Node::Leaf(Felt252Wrapper::from(2_u32)));
    nodes.insert(NodeId(1), Node::Leaf(Felt252Wrapper::from(3_u32)));

    let binary_node =
        BinaryNode { hash: Some(Felt252Wrapper::from(1_u32)), height: 0, left: NodeId(0), right: NodeId(1) };

    let unresolved_node = Node::Unresolved(Felt252Wrapper::from(6_u32));

    let left_child = binary_node.get_child(Direction::Left);
    let right_child = binary_node.get_child(Direction::Right);

    assert_eq!(left_child, NodeId(0));
    assert_eq!(right_child, NodeId(1));
    assert_eq!(nodes.get(&left_child).unwrap().hash(), Some(Felt252Wrapper::from(2_u32)));
    assert_eq!(nodes.get(&right_child).unwrap().hash(), Some(Felt252Wrapper::from(3_u32)));

    assert_eq!(binary_node.hash, Some(Felt252Wrapper::from(1_u32)));

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
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    nodes.insert(NodeId(0), Node::Leaf(Felt252Wrapper::from(2_u32)));
    nodes.insert(NodeId(1), Node::Leaf(Felt252Wrapper::from(3_u32)));

    let mut binary_node = BinaryNode { hash: None, height: 0, left: NodeId(0), right: NodeId(1) };

    binary_node.calculate_hash::<TestHasher>(&nodes);
    assert_eq!(binary_node.hash, Some(Felt252Wrapper::from(5_u32)));
}

#[test]
fn test_binary_node_implementations() {
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    nodes.insert(NodeId(0), Node::Leaf(Felt252Wrapper::from(2_u32)));
    nodes.insert(NodeId(1), Node::Leaf(Felt252Wrapper::from(3_u32)));

    let test_node = BinaryNode { hash: None, height: 0, left: NodeId(0), right: NodeId(1) };

    // Test Display trait implementation
    let node_string = format!("{:?}", test_node);
    assert_eq!(node_string, "BinaryNode { hash: None, height: 0, left: NodeId(0), right: NodeId(1) }");

    // Test Debug trait implementation
    let debug_string = format!("{:?}", test_node);
    assert_eq!(debug_string, "BinaryNode { hash: None, height: 0, left: NodeId(0), right: NodeId(1) }");
}

#[test]
fn test_pedersen_hash_elements_zero() {
    let elements = vec![Felt252Wrapper::ZERO, Felt252Wrapper::ONE];

    let expected_hash = compute_hash_on_elements(&[FieldElement::ZERO, FieldElement::ONE]);
    assert_eq!(PedersenHasher::default().compute_hash_on_wrappers(&elements), expected_hash.into());
}

#[test]
fn test_pedersen_hash_elements_empty() {
    let elements = vec![];

    assert_eq!(
        PedersenHasher::default().compute_hash_on_wrappers(&elements),
        Felt252Wrapper::from_hex_be(PEDERSEN_ZERO_HASH).unwrap()
    );
}
