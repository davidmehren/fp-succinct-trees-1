use bio::data_structures::rank_select::RankSelect;
use bit_vec::BitVec;
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
        let mut bits = BitVec::from_elem(64, false);
        bits.set(5, true);
        bits.set(32, true);
        BPTree {
            bits: BitVec::from_elem(10, false),
            rankselect: RankSelect::new(&bits, 1),
        }
    }
}