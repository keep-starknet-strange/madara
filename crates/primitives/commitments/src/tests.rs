use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use starknet_api::stdlib::collections::HashMap;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::FieldElement;

use super::merkle_patricia_tree::merkle_node::{BinaryNode, Direction, Node, NodeId};

pub const PEDERSEN_ZERO_HASH: &str = "0x49EE3EBA8C1600700EE1B87EB599F16716B0B1022947733551FDE4050CA6804";

#[derive(Default)]
struct TestHasher;

impl HasherT for TestHasher {
    fn hash_bytes(_data: &[u8]) -> Felt252Wrapper {
        unimplemented!()
    }

    fn compute_hash_on_wrappers(_data: &[Felt252Wrapper]) -> Felt252Wrapper {
        unimplemented!()
    }

    fn hash_elements(a: FieldElement, b: FieldElement) -> FieldElement {
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
    assert_eq!(PedersenHasher::compute_hash_on_wrappers(&elements), expected_hash.into());
}

#[test]
fn test_pedersen_hash_elements_empty() {
    let elements = vec![];

    assert_eq!(
        PedersenHasher::compute_hash_on_wrappers(&elements),
        Felt252Wrapper::from_hex_be(PEDERSEN_ZERO_HASH).unwrap()
    );
}

// TODO: add tests to poseidon hasher too
