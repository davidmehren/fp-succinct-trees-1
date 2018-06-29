// Copyright 2018 David Mehren.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! LOUDS succinct tree implementation based on Jacobson (1989) and Arroyuelo et al. (2010)
//!
//! Example
//!
//! ```
//! #[macro_use]
//! extern crate bv;
//! # extern crate fp_succinct_trees_1;
//!
//! # fn main() {
//! use bv::BitVec;
//! use bv::Bits;
//! use fp_succinct_trees_1::common::succinct_tree::SuccinctTree;
//! use fp_succinct_trees_1::trees::louds_tree::LOUDSTree;
//!
//! let bitvec = bit_vec![true, true, false, false];
//! let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
//! assert!(tree.is_leaf(3).unwrap());
//! # }
//! ```

use bincode::{deserialize, serialize};
use bio::data_structures::rank_select::RankSelect;
use bv::{BitVec, Bits};
use common::errors::{EmptyTreeError, InvalidBitvecError, NodeError};
use common::succinct_tree::SuccinctTree;
use failure::{Error, ResultExt};
use id_tree::Tree;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct LOUDSTree {
    rankselect: RankSelect,
}

impl PartialEq for LOUDSTree {
    fn eq(&self, other: &Self) -> bool {
        self.rankselect.bits() == other.rankselect.bits()
    }
}

impl Debug for LOUDSTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "LOUDSTree\n  {{ bits: {:?} }}", self.rankselect.bits())
    }
}

impl SuccinctTree<LOUDSTree> for LOUDSTree {
    /// Checks if a node is a leaf.
    /// # Arguments
    /// * `index` The index of the node to check
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    fn is_leaf(&self, index: u64) -> Result<bool, NodeError> {
        if index >= self.rankselect.bits().bit_len()
            || index == 0
            || (!self.rankselect.bits().get_bit(index) && self.rankselect.bits().get_bit(index - 1))
        {
            Err(NodeError::NotANodeError)
        } else {
            Ok(!self.rankselect.bits().get_bit(index))
        }
    }

    /// Returns the index of the parent of this node
    /// # Arguments
    /// * `index` The index of the node to get the parent of.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `HasNoParentError` If `index` references the root node.
    fn parent(&self, index: u64) -> Result<u64, NodeError> {
        if index >= self.rankselect.bits().bit_len() || index == 0 {
            Err(NodeError::NotANodeError)
        } else if index == 1 {
            Err(NodeError::RootNodeError)
        } else {
            Ok(self
                .prev_0(
                    self.rankselect
                        .select_1(self.rankselect.rank_0(index).unwrap())
                        .unwrap(),
                )
                .unwrap() + 1)
        }
    }

    /// Returns the index of the nodes first child.
    /// # Arguments
    /// * `index` The index of the node to get the first child of.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `NotAParentError` If `index` references a leaf.
    fn first_child(&self, index: u64) -> Result<u64, NodeError> {
        if self.is_leaf(index)? {
            Err(NodeError::NotAParentError)
        } else {
            Ok(self.child(index, 1).unwrap())
        }
    }

    /// Returns the index of the next sibling
    /// # Arguments
    /// * `index` The index of the node to get the next sibling of.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    fn next_sibling(&self, index: u64) -> Result<u64, NodeError> {
        let parent_a = self.parent(index)?;
        let sibling = self
            .rankselect
            .select_0(self.rankselect.rank_0(index - 1).unwrap() + 1)
            .unwrap() + 1;
        let parent_b = self.parent(sibling)?;
        if parent_a == parent_b {
            Ok(sibling)
        } else {
            Err(NodeError::NoSiblingError)
        }
    }

    /// Constructs a LOUDSTree from a IDTree
    /// # Arguments
    /// * `tree` The IDTree which should be converted
    /// # Errors
    /// * `EmptyTreeError` If `tree` does not contain any nodes.
    fn from_id_tree(tree: Tree<i32>) -> Result<Self, EmptyTreeError> {
        let root = match tree.root_node_id() {
            Some(id) => id,
            None => return Err(EmptyTreeError),
        };
        let mut bitvec: BitVec<u8> = BitVec::new_fill(true, 1);
        for node in tree.traverse_level_order(root).unwrap() {
            let child_count = node.children().len();
            for _ in 0..child_count {
                bitvec.push(true);
            }
            bitvec.push(false);
        }
        Ok(Self::from_bitvec(bitvec).unwrap())
    }
}

impl LOUDSTree {
    fn prev_0(&self, index: u64) -> Option<u64> {
        self.rankselect.select_0(self.rankselect.rank_0(index)?)
    }

    fn next_0(&self, index: u64) -> Option<u64> {
        self.rankselect.select_0(self.rankselect.rank_0(index)? + 1)
    }

    pub fn child(&self, index: u64, n: u64) -> Option<u64> {
        Some(
            self.rankselect
                .select_0(self.rankselect.rank_1(index)? + n - 2)? + 1,
        )
    }
    pub fn degree(&self, index: u64) -> Result<u64, NodeError> {
        if self.is_leaf(index)? {
            Ok(0)
        } else {
            // We could just unwrap() here, because invalid indices have been dealt with in is_leaf()
            Ok(self.next_0(index).ok_or(NodeError::NotANodeError)? - index)
        }
    }
    pub fn child_rank(&self, index: u64) -> Option<u64> {
        let y = self
            .rankselect
            .select_1(self.rankselect.rank_0(index - 1)?)?;
        Some(y - self.prev_0(y)?)
    }
    pub fn from_bitvec(bitvec: BitVec<u8>) -> Result<Self, InvalidBitvecError> {
        if !Self::is_valid(&bitvec as &BitVec<u8>) {
            return Err(InvalidBitvecError);
        }
        let superblock_size = Self::calc_superblock_size(bitvec.len());
        Ok(Self {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
        })
    }

    pub fn from_file(path: String) -> Result<Self, Error> {
        let file = fs::read(path).context("Could not read saved tree.")?;
        let tree: Self = deserialize(&file).context("Error while deserializing tree.")?;
        Ok(tree)
    }

    pub fn save_to(&self, path: String) -> Result<(), Error> {
        let encoded = serialize(&self).context("Error while serializing tree.")?;
        let mut file = File::create(path).context("Could not save tree.")?;
        file.write_all(&encoded)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use id_tree::InsertBehavior::{AsRoot, UnderNode};
    use id_tree::{Node, NodeId, TreeBuilder};

    #[test]
    fn new_from_bitvec() {
        let bitvec = bit_vec![true, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            *tree.rankselect.bits(),
            bitvec,
            "BPTree seems to somehow change the bitvector it was created with."
        );
    }

    #[test]
    fn new_from_bitvec_invalid() {
        let bitvec = bit_vec![true, true];
        assert_eq!(
            LOUDSTree::from_bitvec(bitvec.clone()).unwrap_err(),
            InvalidBitvecError
        );
    }

    #[test]
    fn is_leaf() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(tree.is_leaf(3).unwrap());
    }

    #[test]
    fn is_no_leaf() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(!tree.is_leaf(1).unwrap());
    }

    #[test]
    fn is_leaf_wrong_index() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(2).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn is_leaf_wrong_index2() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(4).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn first_child() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(1).unwrap(), 3);
    }

    #[test]
    fn first_child_no_parent() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(3).unwrap_err(), NodeError::NotAParentError);
    }

    #[test]
    fn parent() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(3).unwrap(), 1)
    }

    #[test]
    fn parent_root_node() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(1).unwrap_err(), NodeError::RootNodeError)
    }

    #[test]
    fn parent_no_node() {
        let bitvec = bit_vec![true, true, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(0).unwrap_err(), NodeError::NotANodeError)
    }

    #[test]
    fn next_sibling() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.next_sibling(5).unwrap(), 7);
        assert_eq!(tree.next_sibling(7).unwrap(), 9);
    }

    #[test]
    fn no_next_sibling() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            tree.next_sibling(10).unwrap_err(),
            NodeError::NoSiblingError
        );
    }

    #[test]
    fn degree() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.degree(1).unwrap(), 3);
        assert_eq!(tree.degree(5).unwrap(), 1);
        assert_eq!(tree.degree(9).unwrap(), 0);
    }

    #[test]
    fn child_rank() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.child_rank(9).unwrap(), 2);
        assert_eq!(tree.child_rank(7).unwrap(), 1);
        assert_eq!(tree.child_rank(5).unwrap(), 0);
    }

    #[test]
    fn print() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        let str = format!("{:?}", tree);
        assert_eq!(str, "LOUDSTree\n  { bits: bit_vec![true, true, true, true, false, true, false, true, false, false, false, false] }")
    }

    #[test]
    fn partial_eq() {
        let bitvec_a =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let bitvec_b = bit_vec![true, true, false, false];
        let tree_a = LOUDSTree::from_bitvec(bitvec_a.clone()).unwrap();
        let tree_b = LOUDSTree::from_bitvec(bitvec_a.clone()).unwrap();
        let tree_c = LOUDSTree::from_bitvec(bitvec_b.clone()).unwrap();
        assert_eq!(tree_a, tree_b);
        assert_ne!(tree_a, tree_c)
    }

    #[test]
    fn save_load() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        tree.save_to("testdata/loudstree.testdata".to_string())
            .unwrap();
        let result = LOUDSTree::from_file("testdata/loudstree.testdata".to_string()).unwrap();
        assert_eq!(
            tree, result,
            "The loaded tree is not equal to the original one."
        );
    }

    #[test]
    #[should_panic(expected = "Error while deserializing tree.")]
    fn load_invalid() {
        LOUDSTree::from_file("testdata/bptree_invalid.testdata".to_string()).unwrap();
    }

    #[test]
    fn from_id_tree() {
        let mut id_tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree.insert(Node::new(0), AsRoot).unwrap();
        let child_id = id_tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(3), UnderNode(&child_id)).unwrap();
        let tree = LOUDSTree::from_id_tree(id_tree).unwrap();
        let bitvec = bit_vec![true, true, true, false, true, false, false, false];
        let other_tree = LOUDSTree::from_bitvec(bitvec).unwrap();
        assert_eq!(tree, other_tree)
    }

    #[test]
    fn from_empty_id_tree() {
        let id_tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
        assert_eq!(
            LOUDSTree::from_id_tree(id_tree).unwrap_err(),
            EmptyTreeError
        );
    }
}
