//! Contains constructs for describing the nodes in a Binary Merkle Patricia Tree
//! used by Starknet.
//!
//! For more information about how these Starknet trees are structured, see
//! [`MerkleTree`](super::merkle_tree::MerkleTree).

use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use bitvec::slice::BitSlice;
use scale_codec::{Decode, Encode};
use starknet_api::stdlib::collections::HashMap;

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::HasherT;

/// Id of a Node within the tree
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, scale_info::TypeInfo, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Mutates the given NodeId to be the next one and returns it.
    pub fn next_id(&mut self) -> NodeId {
        self.0 += 1;
        NodeId(self.0)
    }
}

/// A node in a Binary Merkle-Patricia Tree graph.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo, PartialOrd, Ord, Hash)]
pub enum Node {
    /// A node that has not been fetched from storage yet.
    ///
    /// As such, all we know is its hash.
    Unresolved(Felt252Wrapper),
    /// A branch node with exactly two children.
    Binary(BinaryNode),
    /// Describes a path connecting two other nodes.
    Edge(EdgeNode),
    /// A leaf node that contains a value.
    Leaf(Felt252Wrapper),
}

/// Describes the [Node::Binary] variant.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo, PartialOrd, Ord, Hash)]
pub struct BinaryNode {
    /// The hash of this node. Is [None] if the node
    /// has not yet been committed.
    pub hash: Option<Felt252Wrapper>,
    /// The height of this node in the tree.
    pub height: u64,
    /// [Left](Direction::Left) child.
    pub left: NodeId,
    /// [Right](Direction::Right) child.
    pub right: NodeId,
}

/// Node that is an edge.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo, PartialOrd, Ord, Hash)]
pub struct EdgeNode {
    /// The hash of this node. Is [None] if the node
    /// has not yet been committed.
    pub hash: Option<Felt252Wrapper>,
    /// The starting height of this node in the tree.
    pub height: u64,
    /// The path this edge takes.
    pub path: BitVec<u8, Msb0>,
    /// The child of this node.
    pub child: NodeId,
}

/// Describes the direction a child of a [BinaryNode] may have.
///
/// Binary nodes have two children, one left and one right.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Left direction.
    Left,
    /// Right direction.
    Right,
}

impl Direction {
    /// Inverts the [Direction].
    ///
    /// [Left] becomes [Right], and [Right] becomes [Left].
    ///
    /// [Left]: Direction::Left
    /// [Right]: Direction::Right
    pub fn invert(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl From<bool> for Direction {
    fn from(tf: bool) -> Self {
        match tf {
            true => Direction::Right,
            false => Direction::Left,
        }
    }
}

impl From<Direction> for bool {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Left => false,
            Direction::Right => true,
        }
    }
}

impl BinaryNode {
    /// Maps the key's bit at the binary node's height to a [Direction].
    ///
    /// This can be used to check which direction the key describes in the context
    /// of this binary node i.e. which direction the child along the key's path would
    /// take.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the direction of.
    ///
    /// # Returns
    ///
    /// The direction of the key.
    pub fn direction(&self, key: &BitSlice<u8, Msb0>) -> Direction {
        key[self.height as usize].into()
    }

    /// Returns the [Left] or [Right] child.
    ///
    /// [Left]: Direction::Left
    /// [Right]: Direction::Right
    ///
    /// # Arguments
    ///
    /// `direction` - The direction where to get the child from.
    ///
    /// # Returns
    ///
    /// The child in the specified direction.
    pub fn get_child(&self, direction: Direction) -> NodeId {
        match direction {
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }

    /// If possible, calculates and sets its own hash value.
    ///
    /// Does nothing if the hash is already [Some].
    ///
    /// If either child's hash is [None], then the hash cannot
    /// be calculated and it will remain [None].
    pub(crate) fn calculate_hash<H: HasherT>(&mut self, nodes: &HashMap<NodeId, Node>) {
        if self.hash.is_some() {
            return;
        }

        let left = match nodes.get(&self.left) {
            Some(node) => match node.hash() {
                Some(hash) => hash,
                None => unreachable!("subtrees have to be committed first"),
            },
            None => unreachable!("left child not found"),
        };

        let right = match nodes.get(&self.right) {
            Some(node) => match node.hash() {
                Some(hash) => hash,
                None => unreachable!("subtrees have to be committed first"),
            },
            None => unreachable!("right child not found"),
        };

        self.hash = Some(Felt252Wrapper(H::hash_elements(left.0, right.0)));
    }
}

impl Node {
    /// Convenience function which sets the inner node's hash to [None], if
    /// applicable.
    ///
    /// Used to indicate that this node has been mutated.
    pub fn mark_dirty(&mut self) {
        match self {
            Node::Binary(inner) => inner.hash = None,
            Node::Edge(inner) => inner.hash = None,
            _ => {}
        }
    }

    /// Returns true if the node represents an empty node -- this is defined as a node
    /// with the [Felt252Wrapper::ZERO].
    ///
    /// This can occur for the root node in an empty graph.
    pub fn is_empty(&self) -> bool {
        match self {
            Node::Unresolved(hash) => hash == &Felt252Wrapper::ZERO,
            _ => false,
        }
    }

    /// Is the node a binary node.
    pub fn is_binary(&self) -> bool {
        matches!(self, Node::Binary(..))
    }

    /// Convert to node to binary node type (returns None if it's not a binary node).
    pub fn as_binary(&self) -> Option<&BinaryNode> {
        match self {
            Node::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    /// Convert to node to edge node type (returns None if it's not a edge node).
    pub fn as_edge(&self) -> Option<&EdgeNode> {
        match self {
            Node::Edge(edge) => Some(edge),
            _ => None,
        }
    }

    /// Get the hash of a node.
    pub fn hash(&self) -> Option<Felt252Wrapper> {
        match self {
            Node::Unresolved(hash) => Some(*hash),
            Node::Binary(binary) => binary.hash,
            Node::Edge(edge) => edge.hash,
            Node::Leaf(value) => Some(*value),
        }
    }
}

impl EdgeNode {
    /// Returns true if the edge node's path matches the same path given by the key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check if the path matches with the edge node.
    pub fn path_matches(&self, key: &BitSlice<u8, Msb0>) -> bool {
        self.path == key[(self.height as usize)..(self.height + self.path.len() as u64) as usize]
    }

    /// Returns the common bit prefix between the edge node's path and the given key.
    ///
    /// This is calculated with the edge's height taken into account.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the common path from.
    pub fn common_path(&self, key: &BitSlice<u8, Msb0>) -> &BitSlice<u8, Msb0> {
        let key_path = key.iter().skip(self.height as usize);
        let common_length = key_path.zip(self.path.iter()).take_while(|(a, b)| a == b).count();

        &self.path[..common_length]
    }

    /// If possible, calculates and sets its own hash value.
    ///
    /// Does nothing if the hash is already [Some].
    ///
    /// If the child's hash is [None], then the hash cannot
    /// be calculated and it will remain [None].
    pub(crate) fn calculate_hash<H: HasherT>(&mut self, nodes: &HashMap<NodeId, Node>) {
        if self.hash.is_some() {
            return;
        }

        let child = match nodes.get(&self.child) {
            Some(node) => match node.hash() {
                Some(hash) => hash,
                None => unreachable!("subtree has to be committed before"),
            },
            None => unreachable!("child node not found"),
        };

        let mut temp_path = self.path.clone();
        temp_path.force_align();

        let path = Felt252Wrapper::try_from(temp_path.into_vec().as_slice()).unwrap();
        let mut length = [0; 32];
        // Safe as len() is guaranteed to be <= 251
        length[31] = self.path.len() as u8;

        let length = Felt252Wrapper::try_from(&length).unwrap();
        let hash = Felt252Wrapper(H::hash_elements(child.0, path.0) + length.0);
        self.hash = Some(hash);
    }
}
