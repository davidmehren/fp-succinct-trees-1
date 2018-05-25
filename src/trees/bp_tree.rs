use bio::data_structures::rank_select::RankSelect;
use bv::{BitVec, BitsMut};
use common::succinct_tree::SuccinctTree;
use id_tree::Tree;
use std::fmt::Debug;
use std::fmt::Formatter;
use failure::Error;
use std::fmt;

pub struct BPTree {
    bits: BitVec<u32>,
    rankselect: RankSelect,
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
        }
    }
}