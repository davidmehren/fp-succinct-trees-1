use std::fmt::Debug;
use id_tree::Tree;

pub trait SuccinctTree<T>: Debug {
    fn is_leaf(&self, index: u64) -> bool;
    fn parent(&self, index: u64) -> bool;
    fn first_child(&self, index: u64) -> Option<u64>;
    fn next_sibling(&self, index: u64) -> Option<u64>;
    fn from_id_tree(tree: Tree<i32>) -> T;
}