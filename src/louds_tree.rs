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
//! use fp_succinct_trees_1::louds_tree::LOUDSTree;
//!
//! let bitvec = bit_vec![true, true, false, false];
//! let tree: LOUDSTree<i32> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
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
use std::vec::Vec;

pub struct LOUDSTree<L> {
    rankselect: RankSelect,
    labels: Vec<L>,
}

impl<L: PartialEq + Clone + Debug> PartialEq for LOUDSTree<L> {
    fn eq(&self, other: &Self) -> bool {
        self.rankselect.bits() == other.rankselect.bits()
    }
}

impl<L: PartialEq + Clone + Debug> Debug for LOUDSTree<L> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "LOUDSTree\n  {{ bits: {:?} }}", self.rankselect.bits())
    }
}

impl<L: PartialEq + Clone + Debug> SuccinctTree<LOUDSTree<L>, L> for LOUDSTree<L> {
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
    fn from_id_tree(tree: Tree<L>) -> Result<Self, EmptyTreeError> {
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

        let mut l_tree = Self::from_bitvec(bitvec).unwrap();
        for node in tree.traverse_level_order(root).unwrap() {
            l_tree.labels.push((*node.data()).clone());
        }
        Ok(l_tree)
    }

    /// Returns the label for the edge between the parent and the node
    /// # Arguments
    /// * `index` The index of the node to get the label of
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    fn child_label(&self, index: u64) -> Result<&L, NodeError> {
        // child label(x) =
        //L[rank ( (parent(x)) + child rank(x) − 1]
        let parent =
            if index != 1 { self.parent(index)? } else { 0 };
        let child_rank = if index == 1 || self.degree(parent)? == 1 {
            0
        } else {
            self.child_rank(index)
                .ok_or(NodeError::NotANodeError)
                .unwrap()
        };
        let parent_rank = self.rankselect.rank_1(parent).unwrap();
        Ok(self
            .labels
            .get((parent_rank + child_rank - 1) as usize)
            .ok_or(NodeError::NoLabelError)?)
    }

    fn labeled_child(&self, index: u64, label: L) -> Result<u64, NodeError> {
        let child_count = self.degree(index)?;
        for i in 1..=child_count {
            let child_index = self.child(index, i).unwrap();
            let mut label_index = self
                .rankselect
                .rank_0(child_index)
                .ok_or(NodeError::NotANodeError)?;
            if self.is_leaf(child_index).unwrap() {
                label_index -= 1;
            }
            let my_label = &self.labels[label_index as usize];
            if *my_label == label {
                return Ok(child_index);
            }
        }
        Err(NodeError::NoSuchChildError)
    }
}

impl<L: PartialEq + Clone + Debug> LOUDSTree<L> {
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
        if index <= 1 {
            return Some(0);
        }
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
            labels: Vec::with_capacity(bitvec.len() as usize),
            rankselect: RankSelect::new(bitvec, superblock_size as usize),
        })
    }

    pub fn from_file(path: String) -> Result<Self, Error> {
        let file = fs::read(path).context("Could not read saved tree.")?;
        let rankselect: RankSelect = deserialize(&file).context("Error while deserializing tree.")?;
        Ok(Self {
            labels: Vec::with_capacity(rankselect.bits().len() as usize),
            rankselect,
        })
    }

    pub fn save_to(&self, path: String) -> Result<(), Error> {
        let encoded = serialize(&self.rankselect).context("Error while serializing tree.")?;
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
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            *tree.rankselect.bits(),
            bitvec,
            "BPTree seems to somehow change the bitvector it was created with."
        );
    }

    #[test]
    fn new_from_bitvec_invalid() {
        let bitvec = bit_vec![true, true];
        let tree: Result<LOUDSTree<String>, InvalidBitvecError> = LOUDSTree::from_bitvec(bitvec);
        assert_eq!(tree.unwrap_err(), InvalidBitvecError);
    }

    #[test]
    fn save_load() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        tree.save_to("testdata/loudstree.testdata".to_string())
            .unwrap();
        let result: LOUDSTree<String> =
            LOUDSTree::from_file("testdata/loudstree.testdata".to_string()).unwrap();
        assert_eq!(
            tree, result,
            "The loaded tree is not equal to the original one."
        );
    }

    #[test]
    #[should_panic(expected = "Error while deserializing tree.")]
    fn load_invalid() {
        let _tree: LOUDSTree<String> =
            LOUDSTree::from_file("testdata/loudstree_invalid.testdata".to_string()).unwrap();
    }

    #[test]
    fn is_leaf() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(tree.is_leaf(3).unwrap());
    }

    #[test]
    fn is_no_leaf() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(!tree.is_leaf(1).unwrap());
    }

    #[test]
    fn is_leaf_wrong_index() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(2).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn is_leaf_wrong_index2() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(4).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn first_child() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(1).unwrap(), 3);
    }

    #[test]
    fn first_child_no_parent() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(3).unwrap_err(), NodeError::NotAParentError);
    }

    #[test]
    fn parent() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(3).unwrap(), 1)
    }

    #[test]
    fn parent_root_node() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(1).unwrap_err(), NodeError::RootNodeError)
    }

    #[test]
    fn parent_no_node() {
        let bitvec = bit_vec![true, true, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(0).unwrap_err(), NodeError::NotANodeError)
    }

    #[test]
    fn next_sibling() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.next_sibling(5).unwrap(), 7);
        assert_eq!(tree.next_sibling(7).unwrap(), 9);
    }

    #[test]
    fn no_next_sibling() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            tree.next_sibling(10).unwrap_err(),
            NodeError::NoSiblingError
        );
    }

    #[test]
    fn degree() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.degree(1).unwrap(), 3);
        assert_eq!(tree.degree(5).unwrap(), 1);
        assert_eq!(tree.degree(9).unwrap(), 0);
    }

    #[test]
    fn child_rank() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.child_rank(9).unwrap(), 2);
        assert_eq!(tree.child_rank(7).unwrap(), 1);
        assert_eq!(tree.child_rank(5).unwrap(), 0);
        assert_eq!(tree.child_rank(1).unwrap(), 0);
    }

    #[test]
    fn print() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        let str = format!("{:?}", tree);
        assert_eq!(str, "LOUDSTree\n  { bits: bit_vec![true, true, true, true, false, true, false, true, false, false, false, false] }")
    }

    #[test]
    fn partial_eq() {
        let bitvec_a =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let bitvec_b = bit_vec![true, true, false, false];
        let tree_a: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec_a.clone()).unwrap();
        let tree_b: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec_a.clone()).unwrap();
        let tree_c: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec_b.clone()).unwrap();
        assert_eq!(tree_a, tree_b);
        assert_ne!(tree_a, tree_c)
    }

    #[test]
    fn from_id_tree() {
        let mut id_tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree.insert(Node::new(0), AsRoot).unwrap();
        let child_id = id_tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(3), UnderNode(&child_id)).unwrap();
        let tree: LOUDSTree<i32> = LOUDSTree::from_id_tree(id_tree).unwrap();
        let bitvec = bit_vec![true, true, true, false, true, false, false, false];
        let other_tree = LOUDSTree::from_bitvec(bitvec).unwrap();
        assert_eq!(tree, other_tree)
    }

    #[test]
    fn from_empty_id_tree() {
        let id_tree: Tree<String> = TreeBuilder::new().with_node_capacity(5).build();
        let tree: Result<LOUDSTree<String>, EmptyTreeError> = LOUDSTree::from_id_tree(id_tree);
        assert_eq!(tree.unwrap_err(), EmptyTreeError);
    }

    #[test]
    fn child_label() {
        let mut id_tree: Tree<String> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree
            .insert(Node::new(String::from("root")), AsRoot)
            .unwrap();
        let child_id = id_tree
            .insert(
                Node::new(String::from("first_root_child")),
                UnderNode(&root_id),
            )
            .unwrap();
        id_tree
            .insert(Node::new(String::from("leaf")), UnderNode(&child_id))
            .unwrap();
        id_tree
            .insert(
                Node::new(String::from("second_root_child")),
                UnderNode(&root_id),
            )
            .unwrap();
        let bp_tree = LOUDSTree::from_id_tree(id_tree).unwrap();
        assert_eq!(*bp_tree.child_label(1).unwrap(), "root");
        assert_eq!(*bp_tree.child_label(4).unwrap(), "first_root_child");
        assert_eq!(*bp_tree.child_label(6).unwrap(), "second_root_child");
        assert_eq!(*bp_tree.child_label(7).unwrap(), "leaf");
    }

    #[test]
    fn labeled_child() {
        let mut id_tree: Tree<String> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree
            .insert(Node::new(String::from("root")), AsRoot)
            .unwrap();
        let child_id = id_tree
            .insert(
                Node::new(String::from("first_root_child")),
                UnderNode(&root_id),
            )
            .unwrap();
        id_tree
            .insert(Node::new(String::from("leaf")), UnderNode(&child_id))
            .unwrap();
        id_tree
            .insert(
                Node::new(String::from("second_root_child")),
                UnderNode(&root_id),
            )
            .unwrap();
        let louds_tree = LOUDSTree::from_id_tree(id_tree).unwrap();
        assert_eq!(
            louds_tree
                .labeled_child(1, String::from("second_root_child"))
                .unwrap(),
            6
        );
        assert_eq!(
            louds_tree
                .labeled_child(1, String::from("first_root_child"))
                .unwrap(),
            4
        );
        assert_eq!(
            louds_tree.labeled_child(4, String::from("leaf")).unwrap(),
            7
        );
        assert_eq!(
            louds_tree
                .labeled_child(4, String::from("foobar"))
                .unwrap_err(),
            NodeError::NoSuchChildError
        );
    }

    #[test]
    fn nth_child() {
        let bitvec =
            bit_vec![true, true, true, true, false, true, false, true, false, false, false, false];
        let tree: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.child(1, 1).unwrap(), 5);
        assert_eq!(tree.child(1, 2).unwrap(), 7);
        assert_eq!(tree.child(1, 3).unwrap(), 9);
        assert_eq!(tree.child(5, 1).unwrap(), 10);
        assert_eq!(tree.child(7, 1).unwrap(), 11);
        let bitvec2 = bit_vec![true, true, false, true, false, false];
        let tree2: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec2).unwrap();
        assert_eq!(tree2.child(1, 1).unwrap(), 3);
        assert_eq!(tree2.child(3, 1).unwrap(), 5);
        let bitvec3 = bit_vec![true, true, true, false, true, false, false, false];
        let tree3: LOUDSTree<String> = LOUDSTree::from_bitvec(bitvec3).unwrap();
        assert_eq!(tree3.child(1, 1).unwrap(), 4);
        assert_eq!(tree3.child(1, 2).unwrap(), 6);
        assert_eq!(tree3.child(4, 1).unwrap(), 7);
    }
}
