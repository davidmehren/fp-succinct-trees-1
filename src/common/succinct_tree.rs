use std::fmt::Debug;
use id_tree::Tree;
use bit_vec::BitVec;

pub trait SuccinctTree<T>: Debug {
    fn is_leaf(&self, index: u64) -> bool;
    fn parent(&self, index: u64) -> bool;
    fn first_child(&self, index: u64) -> Option<u64>;
    fn next_sibling(&self, index: u64) -> Option<u64>;
    fn from_id_tree(tree: Tree<i32>) -> T;

    ///  Prüft ob ein Bitvector ein gültiger SuccinctTree ist, anhand des gültige Exzesses und
    /// der Anzahl öffnender und schließender Klammern
    fn is_valid(bitvec: BitVec) -> bool {
        let mut excess : u64 = 0;
        for i  in 0..bitvec.len() {
            let x = bitvec.get(i).unwrap();
            if x {
                excess = excess + 1;
            } else {
                excess = excess - 1;
            }
            if excess == 0 && i < bitvec.len() {
                return false
            }
        }
        if excess != 0 {
            return false
        }
        true
    }

}