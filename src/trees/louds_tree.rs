use bincode::{deserialize, serialize};
use bio::data_structures::rank_select::RankSelect;
use bv::{BitVec, Bits};
use common::errors::NodeError;
use common::succinct_tree::SuccinctTree;
use failure::{Error, ResultExt};
use id_tree::Tree;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize)]
pub struct LOUDSTree {
    bits: BitVec<u8>,
    rankselect: RankSelect,
}

impl PartialEq for LOUDSTree {
    fn eq(&self, other: &LOUDSTree) -> bool {
        self.bits == other.bits
    }
}

impl Debug for LOUDSTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl SuccinctTree<LOUDSTree> for LOUDSTree {
    fn is_leaf(&self, index: u64) -> Result<bool, NodeError> {
        if index >= self.bits.bit_len() || index <= 0 {
            Err(NodeError::NotANodeError)
        } else if !self.bits.get_bit(index) && self.bits.get_bit(index - 1) {
            Err(NodeError::NotANodeError)
        } else {
            Ok(!self.bits.get_bit(index))
        }
    }

    fn parent(&self, index: u64) -> Result<u64, NodeError> {
        if index >= self.bits.bit_len() || index == 0 {
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

    fn first_child(&self, index: u64) -> Result<u64, NodeError> {
        if self.is_leaf(index)? {
            Err(NodeError::NotAParentError)
        } else {
            Ok(self.child(index, 1).unwrap())
        }
    }

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

    fn from_id_tree(tree: Tree<i32>) -> Result<LOUDSTree, Error> {
        unimplemented!()
    }
}

impl LOUDSTree {
    fn prev_0(&self, index: u64) -> Option<u64> {
        self.rankselect.select_0(self.rankselect.rank_0(index)?)
    }

    fn prev_1(&self, index: u64) -> Option<u64> {
        self.rankselect.select_1(self.rankselect.rank_1(index)? - 1)
    }

    fn next_0(&self, index: u64) -> Option<u64> {
        self.rankselect.select_0(self.rankselect.rank_0(index)? + 1)
    }

    fn next_1(&self, index: u64) -> Option<u64> {
        self.rankselect.select_1(self.rankselect.rank_1(index)?)
    }
    pub fn child(&self, index: u64, n: u64) -> Option<u64> {
        Some(
            self.rankselect
                .select_0(self.rankselect.rank_1(index)? + n - 2)? + 1,
        )
    }
    pub fn degree(&self, index: u64) -> Option<u64> {
        Some(self.next_0(index)? - index)
    }
    pub fn child_rank(&self, index: u64) -> Option<u64> {
        let y = self
            .rankselect
            .select_1(self.rankselect.rank_0(index - 1)?)?;
        Some(y - self.prev_0(y)?)
    }
    pub fn from_bitvec(bitvec: BitVec<u8>) -> Result<LOUDSTree, Error> {
        if !Self::is_valid(&bitvec as &BitVec<u8>) {
            return Err(format_err!("Bit vector not valid."));
        }
        let superblock_size = Self::calc_superblock_size(bitvec.len());
        Ok(LOUDSTree {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
            bits: bitvec,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bv::BitsMut;

    #[test]
    fn new_from_bitvec() {
        let bitvec = bit_vec![true, false];
        let tree = LOUDSTree::from_bitvec(bitvec.clone()).unwrap();
        assert_eq!(
            tree.bits, bitvec,
            "BPTree seems to somehow change the bitvector it was created with."
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

}
