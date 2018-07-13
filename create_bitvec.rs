extern crate bv;
extern crate rand;
extern crate fp_succinct_trees_1;

use bv::{BitVec, BitsMut};
use fp_succinct_trees_1::bp_tree::*;
use fp_succinct_trees_1::common::succinct_tree::SuccinctTree;
use fp_succinct_trees_1::common::errors::InvalidBitvecError;

fn main() {
    while true {
        let tree: Result<BPTree<i32>, InvalidBitvecError> = BPTree::from_bitvec(generate_bits());
        match tree {
            Ok(tree) => {
                println!("Found {:?}", tree);
                tree.save_to("testdata/bphuge.benchdata".to_string());
                break;
            },
            Err(_) => {}
        }
    }

}



fn generate_bits() -> BitVec<u8> {
    let mut bv: BitVec<u8> = BitVec::new();
    let mut excess = 0;
    for i in 0..10000 {
        let bit = rand::random::<bool>();
        bv.push(bit);
        if bit {
            excess += 1;
        } else { excess -= 1; }
    }
    while excess != 0 {
        if excess < 0 {
            bv.push( true);
            excess += 1;
        } else {
            bv.push(false);
            excess -= 1;
        }
    }
    // println!("Generated bv: {:?}", bv);
    bv
}