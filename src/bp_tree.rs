// Copyright 2018 Kevin Kaßelmann.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! BP succinct tree implementation based on Jacobson (1989), Munro and Raman (2001),
//! Arroyuelo et al. (2010) and Cordova and Navarro (2016).
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
//! use fp_succinct_trees_1::bp_tree::BPTree;
//!
//! let bitvec = bit_vec!(true, true, false, false);
//! let tree: BPTree<i32> = BPTree::from_bitvec(bitvec.clone()).unwrap();
//! assert!(tree.is_leaf(1).unwrap());
//! # }
//! ```

use bincode::{deserialize, serialize};
use bio::data_structures::rank_select::RankSelect;
use bv::BitVec;
use bv::Bits;
use common::errors::EmptyTreeError;
use common::errors::InvalidBitvecError;
use common::errors::NodeError;
use common::min_max::MinMax;
use common::succinct_tree::SuccinctTree;
use failure::{Error, ResultExt};
use id_tree::Node;
use id_tree::NodeId;
use id_tree::Tree;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct BPTree<L: PartialEq + Clone + Debug> {
    labels: Vec<L>,
    rankselect: RankSelect,
    minmax: MinMax,
}

impl<L: PartialEq + Clone + Debug> PartialEq for BPTree<L> {
    fn eq(&self, other: &Self) -> bool {
        self.rankselect.bits() == other.rankselect.bits()
    }
}

impl<L: PartialEq + Clone + Debug> SuccinctTree<BPTree<L>, L> for BPTree<L> {
    /// Checks if a node is a leaf.
    /// # Arguments
    /// * `index` The index of the node to check
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    fn is_leaf(&self, index: u64) -> Result<bool, NodeError> {
        self.is_valid_index(index)?;
        Ok(!self.rankselect.bits().get_bit(index + 1))
    }

    /// Returns the index of the parent of this node
    /// # Arguments
    /// * `index` The index of the node to get the parent of.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `HasNoParentError` If `index` references the root node.
    fn parent(&self, index: u64) -> Result<u64, NodeError> {
        self.is_valid_index(index)?;
        if index == 0 {
            Err(NodeError::HasNoParentError)
        } else {
            Ok(self.minmax.enclose(index)? as u64)
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
            Ok(index + 1)
        }
    }

    /// Returns the index of the next sibling
    /// # Arguments
    /// * `index` The index of the node to get the next sibling of.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `NoSiblingError` If `index` has no further siblings.
    fn next_sibling(&self, index: u64) -> Result<u64, NodeError> {
        let parent_a = self.parent(index)?;
        let sibling = self.minmax.find_close(index)? + 1;
        let parent_b = self.parent(sibling)?;
        if parent_a == parent_b {
            Ok(sibling)
        } else {
            Err(NodeError::NoSiblingError)
        }
    }

    /// Constructs a BPTree from a IDTree
    /// # Arguments
    /// * `tree` The IDTree which should be converted
    /// # Errors
    /// * `EmptyTreeError` If `tree` does not contain any nodes.
    fn from_id_tree(tree: Tree<L>) -> Result<Self, EmptyTreeError> {
        let mut labels: Vec<L> = Vec::new();
        let bitvec = if tree.height() > 0 {
            let root_id: &NodeId = tree.root_node_id().unwrap();
            for node in tree.traverse_pre_order(root_id).unwrap() {
                labels.push(node.data().clone());
            }
            Self::traverse_id_tree_for_bitvec(tree.get(root_id).unwrap(), &tree)
        } else {
            return Err(EmptyTreeError);
        };

        let superblock_size = Self::calc_superblock_size(bitvec.len());
        Ok(Self {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
            minmax: MinMax::new(bitvec.clone(), 1024),
            labels,
        })
    }

    /// Returns the label for the edge between the parent and the node
    /// # Arguments
    /// * `index` The index of the node to get the label of
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `NoLabelError` If `index` does not reference a node with a label.
    fn child_label(&self, index: u64) -> Result<&L, NodeError> {
        self.is_valid_index(index)?;
        self.labels
            .get((self.pre_rank(index).unwrap() - 1) as usize)
            .ok_or(NodeError::NoLabelError)
    }

    /// Returns the child from the specified node with that label
    /// # Arguments
    /// * `index` The index of the node to analyze
    /// * `label` The label which a should have
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    /// * `NoSuchChildError` If there is no child which has this label
    fn labeled_child(&self, index: u64, label: L) -> Result<u64, NodeError> {
        self.is_valid_index(index)?;
        let first_child = self.first_child(index)?;
        if *self.child_label(first_child)? == label {
            return Ok(first_child);
        }
        let mut sibling = first_child;
        while self.next_sibling(first_child).err().is_none() {
            sibling = self.next_sibling(sibling)?;
            if *self.child_label(sibling)? == label {
                return Ok(sibling);
            }
        }
        Err(NodeError::NoSuchChildError)
    }
}

impl<L: PartialEq + Clone + Debug> Debug for BPTree<L> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "BPTree\n  {{ bits: {:?} }}", self.rankselect.bits())
    }
}

impl<L: PartialEq + Clone + Debug> BPTree<L> {
    /// Returns whether the index is valid
    /// # Arguments
    /// * `index` The index which should be valid
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    pub fn is_valid_index(&self, index: u64) -> Result<bool, NodeError> {
        if index >= self.rankselect.bits().len() {
            Err(NodeError::NotANodeError)
        } else {
            Ok(true)
        }
    }

    /// Returns the rank of this index
    /// # Arguments
    /// * `index` The index of the node to get the rank of.
    ///
    pub fn pre_rank(&self, index: u64) -> Option<u64> {
        self.rankselect.rank_1(index)
    }

    /// Returns the select index to this rank
    /// # Arguments
    /// * `rank` The rank of the nodes to get the index of.
    ///
    pub fn pre_select(&self, rank: u64) -> Option<u64> {
        self.rankselect.select_1(rank)
    }

    /// Returns whether the node at `x` is a parent of the node `y`
    /// # Arguments
    /// * `x` The index of the node which should be parent
    /// * `y` The index of the node which should be child
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    pub fn ancestor(&self, x: u64, y: u64) -> Result<bool, NodeError> {
        self.is_valid_index(x)?;
        self.is_valid_index(y)?;
        Ok(x <= y && y <= self.minmax.find_close(x)?)
    }

    /// Returns the depth of the tree at this index
    /// # Arguments
    /// * `index` The index where the depth should be calculated.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    pub fn depth(&self, index: u64) -> Result<u64, NodeError> {
        self.is_valid_index(index)?;
        Ok(self.minmax.excess(index)?)
    }

    /// Returns the size of the subtree from this index
    /// # Arguments
    /// * `index` The index where the subtree size should be calculated.
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    pub fn subtree_size(&self, index: u64) -> Result<u64, NodeError> {
        self.is_valid_index(index)?;
        Ok((self.minmax.find_close(index)? - index + 1) / 2)
    }

    /// Returns a BPTree from a given BitVec
    /// # Arguments
    /// * `bitvec` The BitVec for the specified BPTree
    ///
    pub fn from_bitvec(bitvec: BitVec<u8>) -> Result<Self, InvalidBitvecError> {
        if !Self::is_valid(&bitvec as &BitVec<u8>) {
            return Err(InvalidBitvecError);
        }
        let superblock_size = Self::calc_superblock_size(bitvec.len());
        Ok(Self {
            minmax: MinMax::new(bitvec.clone(), 1024),
            labels: Vec::with_capacity(bitvec.len() as usize),
            rankselect: RankSelect::new(bitvec, superblock_size as usize),
        })
    }

    /// Deserializes a BPTree from a given file
    /// # Arguments
    /// * `path` The path of the file to deserialize
    ///
    pub fn from_file(path: String) -> Result<Self, Error> {
        let file = fs::read(path).context("Could not read saved tree.")?;
        let rankselect: RankSelect = deserialize(&file).context("Error while deserializing tree.")?;
        Ok(Self {
            minmax: MinMax::new(rankselect.bits().clone(), 1024),
            labels: Vec::with_capacity(rankselect.bits().len() as usize),
            rankselect,
        })
    }

    /// Serializes a BPTree to a file
    /// # Arguments
    /// * `path` The path of the file to save to. Will be overwritten if it exists.
    ///
    pub fn save_to(&self, path: String) -> Result<(), Error> {
        let encoded = serialize(&self.rankselect).context("Error while serializing tree.")?;
        let mut file = File::create(path).context("Could not save tree.")?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn traverse_id_tree_for_bitvec(node: &Node<L>, tree: &Tree<L>) -> BitVec<u8> {
        let mut bitvec = BitVec::new();
        bitvec.push(true);
        for child in node.children() {
            let bitvec_rec = Self::traverse_id_tree_for_bitvec(tree.get(child).unwrap(), &tree);
            for bit in 0..bitvec_rec.len() {
                bitvec.push(bitvec_rec.get_bit(bit));
            }
        }
        bitvec.push(false);
        bitvec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use id_tree::InsertBehavior::AsRoot;
    use id_tree::InsertBehavior::UnderNode;
    use id_tree::TreeBuilder;

    #[test]
    fn new_from_bitvec() {
        let bitvec = bit_vec!(true, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            tree.rankselect.bits().clone(),
            bitvec,
            "BPTree seems to somehow change the bitvector it was created with."
        );
    }

    #[test]
    fn new_from_bitvec_invalid() {
        let bitvec = bit_vec!(false, false);
        let tree: Result<BPTree<String>, InvalidBitvecError> = BPTree::from_bitvec(bitvec.clone());
        assert_eq!(tree.unwrap_err(), InvalidBitvecError);
    }

    #[test]
    fn save_load() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        let path = "testdata/bptree.testdata";
        tree.save_to(path.to_string()).unwrap();
        let result = BPTree::from_file(path.to_string()).unwrap();
        assert_eq!(
            tree, result,
            "The loaded tree is not equal to the original one."
        );
    }

    #[test]
    #[should_panic(expected = "Error while deserializing tree.")]
    fn load_invalid() {
        let _tree: BPTree<String> =
            BPTree::from_file("testdata/bptree_invalid.testdata".to_string()).unwrap();
    }

    #[test]
    fn is_leaf() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(tree.is_leaf(1).unwrap());
    }

    #[test]
    fn is_no_leaf() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(!tree.is_leaf(0).unwrap());
    }

    #[test]
    fn is_leaf_wrong_index() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(4).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn parent() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(1).unwrap(), 0);
    }

    #[test]
    fn parent_no_parent() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.parent(0).unwrap_err(), NodeError::HasNoParentError);
    }

    #[test]
    fn first_child() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(0).unwrap(), 1);
    }

    #[test]
    fn first_child_no_parent() {
        let bitvec = bit_vec!(true, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(1).unwrap_err(), NodeError::NotAParentError);
    }

    #[test]
    fn next_sibling() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.next_sibling(1).unwrap(), 3);
    }

    #[test]
    fn next_sibling_no_next_sibling() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.next_sibling(3).unwrap_err(), NodeError::NoSiblingError);
    }

    #[test]
    fn pre_rank() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.pre_rank(1).unwrap(), 2);
        assert_eq!(tree.pre_rank(2).unwrap(), 2);
        assert_eq!(tree.pre_rank(3).unwrap(), 3);
        assert_eq!(tree.pre_rank(4).unwrap(), 3);
        assert_eq!(tree.pre_rank(6), None);
    }

    #[test]
    fn pre_select() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.pre_select(1).unwrap(), 0);
        assert_eq!(tree.pre_select(2).unwrap(), 1);
        assert_eq!(tree.pre_select(3).unwrap(), 3);
        assert_eq!(tree.pre_select(0), None);
        assert_eq!(tree.pre_select(4), None);
    }

    #[test]
    fn ancestor_is_ancestor() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(tree.ancestor(0, 1).unwrap());
        assert!(tree.ancestor(0, 3).unwrap());
    }

    #[test]
    fn ancestor_not_a_ancestor() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(!tree.ancestor(1, 3).unwrap());
        assert!(!tree.ancestor(1, 0).unwrap());
    }

    #[test]
    fn depth() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.depth(0).unwrap(), 1);
        assert_eq!(tree.depth(1).unwrap(), 2);
        assert_eq!(tree.depth(3).unwrap(), 2);
    }

    #[test]
    fn subtree_size() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.subtree_size(0).unwrap(), 3);
        assert_eq!(tree.subtree_size(1).unwrap(), 1);
        assert_eq!(tree.subtree_size(3).unwrap(), 1);
    }

    #[test]
    fn traverse_id_tree_for_bitvec() {
        let bitvec = bit_vec!(true, true, true, false, false, true, false, false);
        let expected_tree: BPTree<i32> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        let mut id_tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree.insert(Node::new(0), AsRoot).unwrap();
        let child_id = id_tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(3), UnderNode(&child_id)).unwrap();
        let tree = BPTree::from_id_tree(id_tree).unwrap();
        assert_eq!(tree, expected_tree);
    }

    #[test]
    fn from_empty_id_tree() {
        let id_tree: Tree<String> = TreeBuilder::new().with_node_capacity(5).build();
        let bp_tree: Result<BPTree<String>, EmptyTreeError> = BPTree::from_id_tree(id_tree);
        assert_eq!(bp_tree.unwrap_err(), EmptyTreeError);
    }

    #[test]
    fn print() {
        let bitvec = bit_vec!(true, true, false, true, false, false);
        let tree: BPTree<String> = BPTree::from_bitvec(bitvec.clone()).unwrap();
        let str = format!("{:?}", tree);
        assert_eq!(
            str,
            "BPTree\n  { bits: bit_vec![true, true, false, true, false, false] }"
        )
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
        let bp_tree = BPTree::from_id_tree(id_tree).unwrap();
        assert_eq!(*bp_tree.child_label(0).unwrap(), "root");
        assert_eq!(*bp_tree.child_label(1).unwrap(), "first_root_child");
        assert_eq!(*bp_tree.child_label(2).unwrap(), "leaf");
        assert_eq!(*bp_tree.child_label(5).unwrap(), "second_root_child");
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
        let bp_tree = BPTree::from_id_tree(id_tree).unwrap();
        assert_eq!(
            bp_tree
                .labeled_child(0, String::from("second_root_child"))
                .unwrap(),
            5
        );
        assert_eq!(
            bp_tree
                .labeled_child(0, String::from("first_root_child"))
                .unwrap(),
            1
        );
        assert_eq!(bp_tree.labeled_child(1, String::from("leaf")).unwrap(), 2);
    }
}
