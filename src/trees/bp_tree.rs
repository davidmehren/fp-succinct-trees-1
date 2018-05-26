use bio::data_structures::rank_select::RankSelect;
use bv::{BitVec, BitsMut};
use common::succinct_tree::SuccinctTree;
use failure::Error;
use id_tree::Tree;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

pub struct BPTree {
    bits: BitVec<u8>,
    rankselect: RankSelect,
    rminmax: String,
}

impl SuccinctTree<BPTree> for BPTree {
    fn is_leaf(&self, index: u64) -> bool {
        unimplemented!()
    }

    fn parent(&self, index: u64) -> bool {
        unimplemented!()
    }

    fn first_child(&self, index: u64) -> Option<u64> {
        unimplemented!()
    }

    fn next_sibling(&self, index: u64) -> Option<u64> {
        unimplemented!()
    }

    fn from_id_tree(tree: Tree<i32>) -> BPTree {
        unimplemented!()
    }
}

impl Debug for BPTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl BPTree {
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
        let superblock_size = ((bitvec.len() as f32).log2().powi(2) / 32.0).ceil();
        Ok(BPTree {
            rankselect: RankSelect::new(bitvec.clone(), superblock_size as usize),
            bits: bitvec,
            rminmax: "foo".to_string(),
        })
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
        let mut bitvec = BitVec::new_fill(false, 2);
        BPTree::from_bitvec(bitvec.clone()).unwrap();
    }
}
