//! This is a gigantic copy pasta from <https://github.com/eqlabs/pathfinder/tree/main/crates/merkle-tree> Thanks to the equilibrium team and whoever else contributed for the code.
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::iter::once;
use core::marker::PhantomData;

use bitvec::prelude::{BitSlice, BitVec, Msb0};
use derive_more::Constructor;
use scale_codec::{Decode, Encode, Error, Input, Output};
use scale_info::build::Fields;
use scale_info::{Path, Type, TypeInfo};
use starknet_api::stdlib::collections::HashMap;

use crate::crypto::merkle_patricia_tree::merkle_node::{BinaryNode, Direction, EdgeNode, Node, NodeId};
use crate::execution::types::Felt252Wrapper;
use crate::traits::hash::HasherT;

/// Wrapper type for a [HashMap<NodeId, Node>] object. (It's not really a wrapper it's a
/// copy of the type but we implement the necessary traits.)
#[derive(Clone, Debug, PartialEq, Eq, Default, Constructor)]
pub struct NodesMapping(pub HashMap<NodeId, Node>);

/// SCALE trait.
impl Encode for NodesMapping {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        // Convert the NodesMapping to Vec<(NodeId, Node)> to be
        // able to use the Encode trait from this type. We implemented it for NodeId, derived it
        // for Node so we can use it for Vec<(NodeId, Node)>.
        let val: Vec<(NodeId, Node)> = self.0.clone().into_iter().collect();
        dest.write(&Encode::encode(&val));
    }
}
/// SCALE trait.
impl Decode for NodesMapping {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        // Convert the NodesMapping to Vec<(NodeId, Node)> to be
        // able to use the Decode trait from this type. We implemented it for NodeId, derived it
        // for Node so we can use it for Vec<(NodeId, Node)>.
        let val: Vec<(NodeId, Node)> =
            Decode::decode(input).map_err(|_| Error::from("Can't get NodesMapping from input buffer."))?;
        Ok(NodesMapping(HashMap::from_iter(val.into_iter())))
    }
}

/// SCALE trait.
impl TypeInfo for NodesMapping {
    type Identity = Self;

    // The type info is saying that the NodesMapping must be seen as an
    // array of bytes.
    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("NodesMapping", module_path!()))
            .composite(Fields::unnamed().field(|f| f.ty::<[u8]>().type_name("NodesMapping")))
    }
}

/// Lightweight representation of [BinaryNode]. Only holds left and right hashes.
#[derive(Debug, Clone, PartialEq, scale_codec::Encode, scale_info::TypeInfo, scale_codec::Decode)]
pub struct BinaryProofNode {
    /// Left hash.
    pub left_hash: Felt252Wrapper,
    /// Right hash.
    pub right_hash: Felt252Wrapper,
}

/// Ligthtweight representation of [EdgeNode]. Only holds its path and its child's hash.
#[derive(Debug, Clone, PartialEq, scale_codec::Encode, scale_info::TypeInfo, scale_codec::Decode)]
pub struct EdgeProofNode {
    /// Path of the node.
    pub path: BitVec<u8, Msb0>,
    /// Hash of the child node.
    pub child_hash: Felt252Wrapper,
}

fn get_proof_node(node: &Node, nodes: &HashMap<NodeId, Node>) -> ProofNode {
    match node {
        Node::Binary(bin) => ProofNode::Binary(BinaryProofNode {
            left_hash: nodes.get(&bin.left).unwrap().hash().expect("Node should be committed"),
            right_hash: nodes.get(&bin.right).unwrap().hash().expect("Node should be committed"),
        }),
        Node::Edge(edge) => ProofNode::Edge(EdgeProofNode {
            path: edge.path.clone(),
            child_hash: nodes.get(&edge.child).unwrap().hash().expect("Node should be committed"),
        }),
        Node::Leaf(_) => panic!("Leaf nodes should not appear in a proof"),
        Node::Unresolved(_) => panic!("Unresolved nodes should not appear in a proof"),
    }
}

/// [ProofNode] s are lightweight versions of their `Node` counterpart.
/// They only consist of [BinaryProofNode] and [EdgeProofNode] because `Leaf`
/// and `Unresolved` nodes should not appear in a proof.
#[derive(Debug, Clone, PartialEq, scale_codec::Encode, scale_info::TypeInfo, scale_codec::Decode)]
pub enum ProofNode {
    /// Binary node.
    Binary(BinaryProofNode),
    /// Edge node.
    Edge(EdgeProofNode),
}

/// A Starknet binary Merkle-Patricia tree with a specific root entry-point and storage.
///
/// This is used to update, mutate and access global Starknet state as well as individual contract
/// states.
///
/// For more information on how this functions internally, see [here](super::merkle_node).
#[derive(Debug, Clone, PartialEq, scale_codec::Encode, scale_info::TypeInfo, scale_codec::Decode)]
pub struct MerkleTree<H: HasherT> {
    root: NodeId,
    nodes: NodesMapping,
    latest_node_id: NodeId,
    _hasher: PhantomData<H>,
}

impl<H: HasherT> MerkleTree<H> {
    /// Less visible initialization for `MerkleTree<T>` as the main entry points should be
    /// [`MerkleTree::<RcNodeStorage>::load`] for persistent trees and [`MerkleTree::empty`] for
    /// transient ones.
    fn new(root: Felt252Wrapper) -> Self {
        let root_node = Node::Unresolved(root);
        let mut nodes_mapping: HashMap<NodeId, Node> = HashMap::new();
        let root_id = NodeId(0); // Assign the appropriate initial node ID here
        nodes_mapping.insert(root_id, root_node);

        Self { root: root_id, nodes: NodesMapping(nodes_mapping), latest_node_id: root_id, _hasher: PhantomData }
    }

    /// Empty tree.
    pub fn empty() -> Self {
        Self::new(Felt252Wrapper::ZERO)
    }

    /// Persists all changes to storage and returns the new root hash.
    ///
    /// Note that the root is reference counted in storage. Committing the
    /// same tree again will therefore increment the count again.
    pub fn commit(&mut self) -> Felt252Wrapper {
        self.commit_mut()
    }

    /// Return the state root.
    pub fn commit_mut(&mut self) -> Felt252Wrapper {
        // Go through the tree, collect dirty nodes, calculate their hashes, and
        // persist them. Take care to increment ref counts of child nodes. Start from
        // the root and traverse the tree.
        self.commit_subtree(&self.root.clone());

        // Unwrap is safe as `commit_subtree` will set the hash.
        let root_hash = self.nodes.0.get(&self.root).unwrap().hash().unwrap();
        root_hash
    }

    /// Persists any changes in this subtree to storage.
    ///
    /// This necessitates recursively calculating the hash of, and
    /// in turn persisting, any changed child nodes. This is necessary
    /// as the parent node's hash relies on its children hashes.
    ///
    /// In effect, the entire subtree gets persisted.
    ///
    /// # Arguments
    ///
    /// * `node` - The top node from the subtree to commit.
    fn commit_subtree(&mut self, node_id: &NodeId) {
        use Node::*;
        let mut nodes = self.nodes.0.clone();
        let node = nodes.get_mut(node_id).unwrap();
        match node {
            Unresolved(_) => { /* Unresolved nodes are already persisted. */ }
            Leaf(_) => { /* storage wouldn't persist these even if we asked. */ }
            Binary(binary) if binary.hash.is_some() => { /* not dirty, already persisted */ }
            Edge(edge) if edge.hash.is_some() => { /* not dirty, already persisted */ }

            Binary(binary) => {
                self.commit_subtree(&binary.left);
                self.commit_subtree(&binary.right);
                // This will succeed as `commit_subtree` will set the child hashes.
                binary.calculate_hash::<H>(&self.nodes.0.clone());
            }

            Edge(edge) => {
                self.commit_subtree(&edge.child);
                // This will succeed as `commit_subtree` will set the child's hash.
                edge.calculate_hash::<H>(&self.nodes.0.clone());
            }
        }

        // Update internal nodes mapping
        self.nodes.0 = nodes.clone();
    }

    /// Sets the value of a key. To delete a key, set the value to [Felt252Wrapper::ZERO].
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to set.
    pub fn set(&mut self, key: &BitSlice<u8, Msb0>, value: Felt252Wrapper) {
        let mut nodes = self.nodes.0.clone();

        if value == Felt252Wrapper::ZERO {
            return self.delete_leaf(key);
        }

        // Changing or inserting a new leaf into the tree will change the hashes
        // of all nodes along the path to the leaf.
        let path = self.traverse(key);
        for node in &path {
            nodes.get_mut(node).unwrap().mark_dirty();
        }

        // There are three possibilities.
        //
        // 1. The leaf exists, in which case we simply change its value.
        //
        // 2. The tree is empty, we insert the new leaf and the root becomes an edge node connecting to it.
        //
        // 3. The leaf does not exist, and the tree is not empty. The final node in the traversal will
        //    be an edge node who's path diverges from our new leaf node's.
        //
        //    This edge must be split into a new subtree containing both the existing edge's child and the
        //    new leaf. This requires an edge followed by a binary node and then further edges to both the
        //    current child and the new leaf. Any of these new edges may also end with an empty path in
        //    which case they should be elided. It depends on the common path length of the current edge
        //    and the new leaf i.e. the split may be at the first bit (in which case there is no leading
        //    edge), or the split may be in the middle (requires both leading and post edges), or the
        //    split may be the final bit (no post edge).
        use Node::*;
        match path.last() {
            Some(node) => {
                let match_node = self.nodes.0.get(node).unwrap();
                let updated: Node = match match_node {
                    Edge(edge) => {
                        let common = edge.common_path(key);

                        // Height of the binary node
                        let branch_height = edge.height as usize + common.len();
                        // Height of the binary node's children
                        let child_height = branch_height + 1;

                        // Path from binary node to new leaf
                        let new_path = key[child_height..].to_bitvec();
                        // Path from binary node to existing child
                        let old_path = edge.path[common.len() + 1..].to_bitvec();

                        // The new leaf branch of the binary node.
                        // (this may be edge -> leaf, or just leaf depending).
                        let new_leaf = Node::Leaf(value);
                        nodes.insert(self.latest_node_id.next_id(), new_leaf);

                        let new = if new_path.is_empty() {
                            self.latest_node_id
                        } else {
                            let new_edge = Node::Edge(EdgeNode {
                                hash: None,
                                height: child_height as u64,
                                path: new_path,
                                child: self.latest_node_id,
                            });
                            nodes.insert(self.latest_node_id.next_id(), new_edge);
                            self.latest_node_id
                        };

                        // The existing child branch of the binary node.
                        let old = if old_path.is_empty() {
                            edge.child
                        } else {
                            let old_edge = Node::Edge(EdgeNode {
                                hash: None,
                                height: child_height as u64,
                                path: old_path,
                                child: edge.child,
                            });
                            nodes.insert(self.latest_node_id.next_id(), old_edge);
                            self.latest_node_id
                        };

                        let new_direction = Direction::from(key[branch_height]);
                        let (left, right) = match new_direction {
                            Direction::Left => (new, old),
                            Direction::Right => (old, new),
                        };

                        let branch = Node::Binary(BinaryNode { hash: None, height: branch_height as u64, left, right });
                        nodes.insert(self.latest_node_id.next_id(), branch.clone());

                        // We may require an edge leading to the binary node.
                        if common.is_empty() {
                            branch
                        } else {
                            let edge = Node::Edge(EdgeNode {
                                hash: None,
                                height: edge.height,
                                path: common.to_bitvec(),
                                child: self.latest_node_id,
                            });
                            nodes.insert(self.latest_node_id.next_id(), edge.clone());
                            edge
                        }
                    }
                    // Leaf exists, we replace its value.
                    Leaf(_) => {
                        let leaf = Node::Leaf(value);
                        nodes.insert(self.latest_node_id.next_id(), leaf.clone());
                        leaf
                    }
                    Unresolved(_) | Binary(_) => {
                        unreachable!("The end of a traversion cannot be unresolved or binary")
                    }
                };

                // node.swap(&Box::new(updated));
                nodes.insert(*node, updated);
                nodes.insert(self.latest_node_id, self.nodes.0.get(node).unwrap().clone());
            }
            None => {
                // Getting no travel nodes implies that the tree is empty.
                //
                // Create a new leaf node with the value, and the root becomes
                // an edge node connecting to the leaf.
                let leaf = Node::Leaf(value);
                nodes.insert(self.latest_node_id.next_id(), leaf);
                let edge =
                    Node::Edge(EdgeNode { hash: None, height: 0, path: key.to_bitvec(), child: self.latest_node_id });
                nodes.insert(self.latest_node_id.next_id(), edge);

                self.root = self.latest_node_id;
            }
        }

        // Updates self nodes mapping
        self.nodes.0 = nodes;
    }

    /// Deletes a leaf node from the tree.
    ///
    /// This is not an external facing API; the functionality is instead accessed by calling
    /// [`MerkleTree::set`] with value set to [`Felt252Wrapper::ZERO`].
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete.
    fn delete_leaf(&mut self, key: &BitSlice<u8, Msb0>) {
        let mut nodes = self.nodes.0.clone();
        // Algorithm explanation:
        //
        // The leaf's parent node is either an edge, or a binary node.
        // If it's an edge node, then it must also be deleted. And its parent
        // must be a binary node. In either case we end up with a binary node
        // who's one child is deleted. This changes the binary to an edge node.
        //
        // Note that its possible that there is no binary node -- if the resulting tree would be empty.
        //
        // This new edge node may need to merge with the old binary node's parent node
        // and other remaining child node -- if they're also edges.
        //
        // Then we are done.
        let path = self.traverse(key);

        // Do nothing if the leaf does not exist.
        match path.last() {
            Some(node) => match nodes.get(node).unwrap() {
                Node::Leaf(_) => {}
                _ => return,
            },
            None => return,
        }

        // All hashes along the path will become invalid (if they aren't deleted).
        for node in &path {
            nodes.get_mut(node).unwrap().mark_dirty();
        }

        // Go backwards until we hit a branch node.
        let mut node_iter = path.into_iter().rev().skip_while(|node| !self.nodes.0.get(node).unwrap().is_binary());

        match node_iter.next() {
            Some(node) => {
                let new_edge = {
                    let node = nodes.get_mut(&node).unwrap();
                    // This node must be a binary node due to the iteration condition.
                    let binary = node.as_binary().cloned().unwrap();
                    // Create an edge node to replace the old binary node
                    // i.e. with the remaining child (note the direction invert),
                    //      and a path of just a single bit.
                    let direction = binary.direction(key).invert();
                    let child = binary.get_child(direction.clone());
                    let path = once(bool::from(direction)).collect::<BitVec<_, _>>();
                    let mut edge = EdgeNode { hash: None, height: binary.height, path, child };

                    // Merge the remaining child if it's an edge.
                    self.merge_edges(&mut edge);

                    edge
                };
                // Replace the old binary node with the new edge node.
                // node.swap(&Box::new(Node::Edge(new_edge)));
                nodes.insert(node, Node::Edge(new_edge));
                nodes.insert(self.latest_node_id, nodes.get(&node).unwrap().clone());
            }
            None => {
                // We reached the root without a hitting binary node. The new tree
                // must therefore be empty.
                self.root = NodeId(0);
                return;
            }
        };

        // Check the parent of the new edge. If it is also an edge, then they must merge.
        if let Some(node) = node_iter.next() {
            if let Node::Edge(edge) = nodes.get_mut(&node).unwrap() {
                self.merge_edges(edge);
            }
        }
    }

    /// Returns the value stored at key, or `None` if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to get.
    ///
    /// # Returns
    ///
    /// The value of the key.
    pub fn get(&self, key: &BitSlice<u8, Msb0>) -> Option<Felt252Wrapper> {
        self.traverse(key).last().and_then(|node| match self.nodes.0.get(node).unwrap() {
            Node::Leaf(value) if !value.eq(&Felt252Wrapper::ZERO) => Some(*value),
            _ => None,
        })
    }

    /// Generates a merkle-proof for a given `key`.
    ///
    /// Returns vector of [`ProofNode`] which form a chain from the root to the key,
    /// if it exists, or down to the node which proves that the key does not exist.
    ///
    /// The nodes are returned in order, root first.
    ///
    /// Verification is performed by confirming that:
    ///   1. the chain follows the path of `key`, and
    ///   2. the hashes are correct, and
    ///   3. the root hash matches the known root
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the merkle proof of.
    ///
    /// # Returns
    ///
    /// The merkle proof and all the child nodes hashes.
    pub fn get_proof(&self, key: &BitSlice<u8, Msb0>) -> Vec<ProofNode> {
        let mut nodes = self.traverse(key);

        // Return an empty list if tree is empty.
        let node = match nodes.last() {
            Some(node) => node,
            None => return Vec::new(),
        };

        // A leaf node is redundant data as the information for it is already contained in the previous
        // node.
        if matches!(self.nodes.0.get(node).unwrap(), Node::Leaf(_)) {
            nodes.pop();
        }

        nodes
            .iter()
            .map(|node| match self.nodes.0.get(node).unwrap() {
                Node::Binary(bin) => get_proof_node(&Node::Binary(bin.clone()), &self.nodes.0),
                Node::Edge(edge) => get_proof_node(&Node::Edge(edge.clone()), &self.nodes.0),
                _ => unreachable!(),
            })
            .collect()
    }

    /// Traverses from the current root towards the destination [Leaf](Node::Leaf) node.
    /// Returns the list of nodes along the path.
    ///
    /// If the destination node exists, it will be the final node in the list.
    ///
    /// This means that the final node will always be either a the destination [Leaf](Node::Leaf)
    /// node, or an [Edge](Node::Edge) node who's path suffix does not match the leaf's path.
    ///
    /// The final node can __not__ be a [Binary](Node::Binary) node since it would always be
    /// possible to continue on towards the destination. Nor can it be an
    /// [Unresolved](Node::Unresolved) node since this would be resolved to check if we can
    /// travel further.
    ///
    /// # Arguments
    ///
    /// * `dst` - The node to get to.
    ///
    /// # Returns
    ///
    /// The list of nodes along the path.
    fn traverse(&self, dst: &BitSlice<u8, Msb0>) -> Vec<NodeId> {
        if self.nodes.0.get(&self.root).unwrap().is_empty() {
            return Vec::new();
        }

        let mut current = self.root;
        #[allow(unused_variables)]
        let mut height = 0;
        let mut nodes = Vec::new();
        loop {
            use Node::*;

            let current_tmp = self.nodes.0.get(&current).unwrap().clone();

            let next = match current_tmp {
                Unresolved(_hash) => panic!("Resolve is useless"),
                Binary(binary) => {
                    nodes.push(current);
                    let next = binary.direction(dst);
                    let next = binary.get_child(next);
                    height += 1;
                    next
                }
                Edge(edge) if edge.path_matches(dst) => {
                    nodes.push(current);
                    height += edge.path.len();
                    edge.child
                }
                Leaf(_) | Edge(_) => {
                    nodes.push(current);
                    return nodes;
                }
            };

            current = next;
        }
    }

    /// This is a convenience function which merges the edge node with its child __iff__ it is also
    /// an edge.
    ///
    /// Does nothing if the child is not also an edge node.
    ///
    /// This can occur when mutating the tree (e.g. deleting a child of a binary node), and is an
    /// illegal state (since edge nodes __must be__ maximal subtrees).
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent node to merge the child with.
    fn merge_edges(&self, parent: &mut EdgeNode) {
        let resolved_child = match self.nodes.0.get(&parent.child).unwrap().borrow() {
            Node::Unresolved(_hash) => panic!("Resolve is useless"),
            other => other.clone(),
        };

        if let Some(child_edge) = resolved_child.as_edge().cloned() {
            parent.path.extend_from_bitslice(&child_edge.path);
            parent.child = child_edge.child;
        }
    }
}
