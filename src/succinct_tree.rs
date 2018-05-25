pub trait SuccinctTree {
    fn is_leaf(index: u64) -> bool;
    fn parent(index: u64) -> bool;
    fn first_child(index: u64) -> Option<u64>;
    fn next_sibling(index: u64) -> Option<u64>;
    fn from_id_tree(tree: Tree<i32>) -> Box<SuccinctTree>;
}