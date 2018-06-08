use bincode::{deserialize, serialize};
use bio::data_structures::rank_select::RankSelect;
use bv::Bits;
use bv::{BitVec, BitsMut};
use common::errors::NodeError;
use common::succinct_tree::SuccinctTree;
use failure::{Error, ResultExt};
use id_tree::InsertBehavior::AsRoot;
use id_tree::InsertBehavior::UnderNode;
use id_tree::Node;
use id_tree::NodeId;
use id_tree::Tree;
use id_tree::TreeBuilder;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct BPTree {
    bits: BitVec<u8>,
    rankselect: RankSelect,
    rminmax: String,
}

impl PartialEq for BPTree {
    fn eq(&self, other: &BPTree) -> bool {
        self.bits == other.bits
    }
}

impl SuccinctTree<BPTree> for BPTree {
    /// Checks if a node is a leaf.
    /// # Arguments
    /// * `index` The index of the node to check
    /// # Errors
    /// * `NotANodeError` If `index` does not reference a node.
    fn is_leaf(&self, index: u64) -> Result<bool, NodeError> {
        if index >= self.bits.bit_len() {
            Err(NodeError::NotANodeError)
        } else {
            Ok(!self.bits.get_bit(index + 1))
        }
    }

    fn parent(&self, index: u64) -> Result<bool, NodeError> {
        unimplemented!()
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

    fn next_sibling(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!()
    }

    fn from_id_tree(tree: Tree<i32>) -> Result<BPTree, Error> {
        // TODO: This should not panic on an empty tree
        let root_id = tree.root_node_id().unwrap();
        let bitvec = BPTree::traverse_id_tree_for_bitvec(tree.get(root_id).unwrap(), &tree);

        let superblock_size = BPTree::calc_superblock_size(bitvec.len());
        Ok(BPTree {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
            bits: bitvec,
            rminmax: "foo".to_string(),
        })
    }
}

impl Debug for BPTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.bits.fmt(f)
    }
}

impl BPTree {
    fn pre_rank(&self, index: u64) {
        self.rankselect.rank_1(index);
    }

    fn pre_select(&self, index: u64) {
        self.rankselect.select_1(index);
    }

    pub fn stub_create() -> BPTree {
        let mut bits: BitVec<u8> = BitVec::new_fill(false, 64);
        bits.set_bit(5, true);
        bits.set_bit(32, true);
        BPTree {
            bits: BitVec::new_fill(false, 10),
            rankselect: RankSelect::new(bits, 1),
            rminmax: "foo".to_string(),
        }
    }

    pub fn from_bitvec(bitvec: BitVec<u8>) -> Result<BPTree, Error> {
        if !Self::is_valid(&bitvec as &BitVec<u8>) {
            return Err(format_err!("Bit vector not valid."));
        }
        let superblock_size = BPTree::calc_superblock_size(bitvec.len());
        Ok(BPTree {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
            bits: bitvec,
            rminmax: "foo".to_string(),
        })
    }

    pub fn from_file(path: String) -> Result<BPTree, Error> {
        let file = fs::read(path).context("Could not read saved tree.")?;
        let tree: BPTree = deserialize(&file).context("Error while deserializing tree.")?;
        Ok(tree)
    }

    pub fn save_to(&self, path: String) -> Result<(), Error> {
        let encoded = serialize(&self).context("Error while serializing tree.")?;
        let mut file = File::create(path).context("Could not save tree.")?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn calc_superblock_size(length: u64) -> f32 {
        ((length as f32).log2().powi(2) / 32.0).ceil()
    }

    fn traverse_id_tree_for_bitvec(node: &Node<i32>, ref tree: &Tree<i32>) -> BitVec<u8> {
        let mut bitvec = BitVec::new();
        bitvec.push(true);
        for child in node.children() {
            let bitvec_rec = BPTree::traverse_id_tree_for_bitvec(tree.get(child).unwrap(), &tree);
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

    #[test]
    fn new_from_bitvec() {
        let mut bitvec = BitVec::new_fill(false, 2);
        bitvec.set_bit(0, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            tree.bits, bitvec,
            "BPTree seems to somehow change the bitvector it was created with."
        );
    }

    #[test]
    #[should_panic(expected = "ErrorMessage { msg: \"Bit vector not valid.\" }")]
    fn new_from_bitvec_invalid() {
        let bitvec = BitVec::new_fill(false, 2);
        BPTree::from_bitvec(bitvec.clone()).unwrap();
    }

    #[test]
    fn save_load() {
        let tree = BPTree::stub_create();
        tree.save_to("testdata/bptree.testdata".to_string())
            .unwrap();
        let result = BPTree::from_file("testdata/bptree.testdata".to_string()).unwrap();
        assert_eq!(
            tree, result,
            "The loaded tree is not equal to the original one."
        );
    }

    #[test]
    #[should_panic(expected = "Error while deserializing tree.")]
    fn load_invaild() {
        BPTree::from_file("testdata/bptree_invalid.testdata".to_string()).unwrap();
    }

    #[test]
    fn is_leaf() {
        let mut bitvec = BitVec::new_fill(false, 4);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(tree.is_leaf(1).unwrap());
    }

    #[test]
    fn is_no_leaf() {
        let mut bitvec = BitVec::new_fill(false, 4);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert!(!tree.is_leaf(0).unwrap());
    }

    #[test]
    fn is_leaf_wrong_index() {
        let mut bitvec = BitVec::new_fill(false, 4);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.is_leaf(4).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn first_child() {
        let mut bitvec = BitVec::new_fill(false, 4);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(0).unwrap(), 1);
    }

    #[test]
    fn first_child_no_parent() {
        let mut bitvec = BitVec::new_fill(false, 4);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        let tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(tree.first_child(1).unwrap_err(), NodeError::NotAParentError);
    }

    #[test]
    fn traverse_id_tree_for_bitvec() {
        let mut bitvec = BitVec::new_fill(false, 8);
        bitvec.set_bit(0, true);
        bitvec.set_bit(1, true);
        bitvec.set_bit(2, true);
        bitvec.set_bit(5, true);
        let expected_tree = BPTree::from_bitvec(bitvec.clone()).unwrap();
        let mut id_tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
        let root_id: NodeId = id_tree.insert(Node::new(0), AsRoot).unwrap();
        let child_id = id_tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();
        id_tree.insert(Node::new(3), UnderNode(&child_id)).unwrap();

        let tree = BPTree::from_id_tree(id_tree).unwrap_or(BPTree::stub_create());
        assert_eq!(tree, expected_tree);
    }
}
