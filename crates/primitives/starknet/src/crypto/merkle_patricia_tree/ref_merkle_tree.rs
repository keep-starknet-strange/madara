//! This is a gigantic copy pasta from <https://github.com/eqlabs/pathfinder/tree/main/crates/merkle-tree> Thanks to the equilibrium team and whoever else contributed for the code.
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::iter::once;
use core::marker::PhantomData;

use bitvec::prelude::{BitSlice, BitVec, Msb0};

use crate::crypto::merkle_patricia_tree::ref_merkle_node::{BinaryNode, Direction, EdgeNode, Node};
use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::HasherT;

/// Lightweight representation of [BinaryNode]. Only holds left and right hashes.
#[derive(Debug, PartialEq, Eq)]
pub struct BinaryProofNode {
    /// Left hash.
    pub left_hash: Felt252Wrapper,
    /// Right hash.
    pub right_hash: Felt252Wrapper,
}

impl From<&BinaryNode> for ProofNode {
    fn from(bin: &BinaryNode) -> Self {
        Self::Binary(BinaryProofNode {
            left_hash: bin.left.borrow().hash().expect("Node should be committed"),
            right_hash: bin.right.borrow().hash().expect("Node should be committed"),
        })
    }
}

/// Ligthtweight representation of [EdgeNode]. Only holds its path and its child's hash.
#[derive(Debug, PartialEq, Eq)]
pub struct EdgeProofNode {
    /// Path of the node.
    pub path: BitVec<u8, Msb0>,
    /// Hash of the child node.
    pub child_hash: Felt252Wrapper,
}

impl From<&EdgeNode> for ProofNode {
    fn from(edge: &EdgeNode) -> Self {
        Self::Edge(EdgeProofNode {
            path: edge.path.clone(),
            child_hash: edge.child.borrow().hash().expect("Node should be committed"),
        })
    }
}

/// [ProofNode] s are lightweight versions of their `Node` counterpart.
/// They only consist of [BinaryProofNode] and [EdgeProofNode] because `Leaf`
/// and `Unresolved` nodes should not appear in a proof.
#[derive(Debug, PartialEq, Eq)]
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
/// For more information on how this functions internally, see [here](super::ref_merkle_tree).
#[derive(Debug, Clone)]
pub struct RefMerkleTree<H: HasherT> {
    root: Rc<RefCell<Node>>,
    _hasher: PhantomData<H>,
}

impl<H: HasherT> RefMerkleTree<H> {
    /// Less visible initialization for `MerkleTree<T>` as the main entry points should be
    /// [`MerkleTree::<RcNodeStorage>::load`] for persistent trees and [`MerkleTree::empty`] for
    /// transient ones.
    fn new(root: Felt252Wrapper) -> Self {
        let root_node = Rc::new(RefCell::new(Node::Unresolved(root)));
        Self { root: root_node, _hasher: PhantomData }
    }

    /// Empty tree.
    pub fn empty() -> Self {
        Self::new(Felt252Wrapper::ZERO)
    }

    /// Persists all changes to storage and returns the new root hash.
    ///
    /// Note that the root is reference counted in storage. Committing the
    /// same tree again will therefore increment the count again.
    pub fn commit(mut self) -> Felt252Wrapper {
        self.commit_mut()
    }
    /// Return the state root.
    pub fn commit_mut(&mut self) -> Felt252Wrapper {
        // Go through tree, collect dirty nodes, calculate their hashes and
        // persist them. Take care to increment ref counts of child nodes. So in order
        // to do this correctly, will have to start back-to-front.
        Self::commit_subtree(&mut self.root.borrow_mut());
        // unwrap is safe as `commit_subtree` will set the hash.
        self.root.borrow().hash().unwrap()
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
    fn commit_subtree(node: &mut Node) {
        use Node::*;
        match node {
            Unresolved(_) => { /* Unresolved nodes are already persisted. */ }
            Leaf(_) => { /* storage wouldn't persist these even if we asked. */ }
            Binary(binary) if binary.hash.is_some() => { /* not dirty, already persisted */ }
            Edge(edge) if edge.hash.is_some() => { /* not dirty, already persisted */ }

            Binary(binary) => {
                Self::commit_subtree(&mut binary.left.borrow_mut());
                Self::commit_subtree(&mut binary.right.borrow_mut());
                // This will succeed as `commit_subtree` will set the child hashes.
                binary.calculate_hash::<H>();
            }

            Edge(edge) => {
                Self::commit_subtree(&mut edge.child.borrow_mut());
                // This will succeed as `commit_subtree` will set the child's hash.
                edge.calculate_hash::<H>();
            }
        }
    }

    /// Sets the value of a key. To delete a key, set the value to [Felt252Wrapper::ZERO].
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to set.
    pub fn set(&mut self, key: &BitSlice<u8, Msb0>, value: Felt252Wrapper) {
        if value == Felt252Wrapper::ZERO {
            return self.delete_leaf(key);
        }

        // Changing or inserting a new leaf into the tree will change the hashes
        // of all nodes along the path to the leaf.
        let path = self.traverse(key);
        for node in &path {
            node.borrow_mut().mark_dirty();
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
                let updated = match &*node.borrow() {
                    Edge(edge) => {
                        let common = edge.common_path(key);

                        // Height of the binary node
                        let branch_height = edge.height + common.len();
                        // Height of the binary node's children
                        let child_height = branch_height + 1;

                        // Path from binary node to new leaf
                        let new_path = key[child_height..].to_bitvec();
                        // Path from binary node to existing child
                        let old_path = edge.path[common.len() + 1..].to_bitvec();

                        // The new leaf branch of the binary node.
                        // (this may be edge -> leaf, or just leaf depending).
                        let new_leaf = Node::Leaf(value);
                        let new = if new_path.is_empty() {
                            Rc::new(RefCell::new(new_leaf))
                        } else {
                            let new_edge = Node::Edge(EdgeNode {
                                hash: None,
                                height: child_height,
                                path: new_path,
                                child: Rc::new(RefCell::new(new_leaf)),
                            });
                            Rc::new(RefCell::new(new_edge))
                        };

                        // The existing child branch of the binary node.
                        let old = if old_path.is_empty() {
                            edge.child.clone()
                        } else {
                            let old_edge = Node::Edge(EdgeNode {
                                hash: None,
                                height: child_height,
                                path: old_path,
                                child: edge.child.clone(),
                            });
                            Rc::new(RefCell::new(old_edge))
                        };

                        let new_direction = Direction::from(key[branch_height]);
                        let (left, right) = match new_direction {
                            Direction::Left => (new, old),
                            Direction::Right => (old, new),
                        };

                        let branch = Node::Binary(BinaryNode { hash: None, height: branch_height, left, right });

                        // We may require an edge leading to the binary node.
                        if common.is_empty() {
                            branch
                        } else {
                            Node::Edge(EdgeNode {
                                hash: None,
                                height: edge.height,
                                path: common.to_bitvec(),
                                child: Rc::new(RefCell::new(branch)),
                            })
                        }
                    }
                    // Leaf exists, we replace its value.
                    Leaf(_) => Node::Leaf(value),
                    Unresolved(_) | Binary(_) => {
                        unreachable!("The end of a traversion cannot be unresolved or binary")
                    }
                };

                node.swap(&RefCell::new(updated));
            }
            None => {
                // Getting no travel nodes implies that the tree is empty.
                //
                // Create a new leaf node with the value, and the root becomes
                // an edge node connecting to the leaf.
                let leaf = Node::Leaf(value);
                let edge = Node::Edge(EdgeNode {
                    hash: None,
                    height: 0,
                    path: key.to_bitvec(),
                    child: Rc::new(RefCell::new(leaf)),
                });

                self.root = Rc::new(RefCell::new(edge));
            }
        }
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
            Some(node) => match &*node.borrow() {
                Node::Leaf(_) => {}
                _ => return,
            },
            None => return,
        }

        // All hashes along the path will become invalid (if they aren't deleted).
        for node in &path {
            node.borrow_mut().mark_dirty();
        }

        // Go backwards until we hit a branch node.
        let mut node_iter = path.into_iter().rev().skip_while(|node| !node.borrow().is_binary());

        match node_iter.next() {
            Some(node) => {
                let new_edge = {
                    // This node must be a binary node due to the iteration condition.
                    let binary = node.borrow().as_binary().cloned().unwrap();
                    // Create an edge node to replace the old binary node
                    // i.e. with the remaining child (note the direction invert),
                    //      and a path of just a single bit.
                    let direction = binary.direction(key).invert();
                    let child = binary.get_child(direction);
                    let path = once(bool::from(direction)).collect::<BitVec<_, _>>();
                    let mut edge = EdgeNode { hash: None, height: binary.height, path, child };

                    // Merge the remaining child if it's an edge.
                    self.merge_edges(&mut edge);

                    edge
                };
                // Replace the old binary node with the new edge node.
                node.swap(&RefCell::new(Node::Edge(new_edge)));
            }
            None => {
                // We reached the root without a hitting binary node. The new tree
                // must therefore be empty.
                self.root = Rc::new(RefCell::new(Node::Unresolved(Felt252Wrapper::ZERO)));
                return;
            }
        };

        // Check the parent of the new edge. If it is also an edge, then they must merge.
        if let Some(node) = node_iter.next() {
            if let Node::Edge(edge) = &mut *node.borrow_mut() {
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
        self.traverse(key).last().and_then(|node| match &*node.borrow() {
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
        if matches!(&*node.borrow(), Node::Leaf(_)) {
            nodes.pop();
        }

        nodes
            .iter()
            .map(|node| match &*node.borrow() {
                Node::Binary(bin) => ProofNode::from(bin),
                Node::Edge(edge) => ProofNode::from(edge),
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
    fn traverse(&self, dst: &BitSlice<u8, Msb0>) -> Vec<Rc<RefCell<Node>>> {
        if self.root.borrow().is_empty() {
            return Vec::new();
        }

        let mut current = self.root.clone();
        #[allow(unused_variables)]
        let mut height = 0;
        let mut nodes = Vec::new();
        loop {
            use Node::*;

            let current_tmp = current.borrow().clone();

            let next = match current_tmp {
                Unresolved(_hash) => panic!("Resolve is useless"),
                Binary(binary) => {
                    nodes.push(current.clone());
                    let next = binary.direction(dst);
                    let next = binary.get_child(next);
                    height += 1;
                    next
                }
                Edge(edge) if edge.path_matches(dst) => {
                    nodes.push(current.clone());
                    height += edge.path.len();
                    edge.child.clone()
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
        let resolved_child = match &*parent.child.borrow() {
            Node::Unresolved(_hash) => panic!("Resolve is useless"),
            other => other.clone(),
        };

        if let Some(child_edge) = resolved_child.as_edge().cloned() {
            parent.path.extend_from_bitslice(&child_edge.path);
            parent.child = child_edge.child;
        }
    }
}
