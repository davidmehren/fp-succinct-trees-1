#[macro_use]
extern crate criterion;
extern crate bv;
extern crate fp_succinct_trees_1;
extern crate id_tree;

use criterion::Criterion;
use criterion::Fun;
use fp_succinct_trees_1::bp_tree::BPTree;
use fp_succinct_trees_1::common::succinct_tree::SuccinctTree;
use fp_succinct_trees_1::louds_tree::LOUDSTree;
use id_tree::InsertBehavior::*;
use id_tree::Node;
use id_tree::NodeId;
use id_tree::Tree;
use id_tree::TreeBuilder;

fn create_bench_tree() -> Tree<i32> {
    let mut tree: Tree<i32> = TreeBuilder::new().with_node_capacity(5).build();
    let root_id: NodeId = tree.insert(Node::new(0), AsRoot).unwrap();
    let child1_id: NodeId = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    let child4_id = tree.insert(Node::new(1), UnderNode(&child1_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child4_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child1_id)).unwrap();
    let child2_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child2_id)).unwrap();
    let child5_id = tree.insert(Node::new(1), UnderNode(&child2_id)).unwrap();
    let child6_id = tree.insert(Node::new(1), UnderNode(&child5_id)).unwrap();
    let child7_id = tree.insert(Node::new(1), UnderNode(&child6_id)).unwrap();
    let child8_id = tree.insert(Node::new(1), UnderNode(&child7_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child8_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child8_id)).unwrap();
    let child3_id = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
    tree.insert(Node::new(1), UnderNode(&child3_id)).unwrap();
    tree
}

fn create_bench_louds() -> LOUDSTree<i32> {
    LOUDSTree::from_file("testdata/loudshuge.benchdata".to_string()).unwrap()
}

fn create_bench_bp() -> BPTree<i32> {
    BPTree::from_file("testdata/bphuge.benchdata".to_string()).unwrap()
}

fn compare_load_huge_tree(c: &mut Criterion) {
    let louds = Fun::new("LOUDS from file", |b, i| b.iter(|| create_bench_louds()));
    let bp = Fun::new("BP from file", |b, i| {
        b.iter(|| create_bench_bp());
    });
    c.bench_functions("Load huge trees", vec![louds, bp], 0);
}

fn create_bench_idtree(c: &mut Criterion) {
    let tree = create_bench_tree();
    c.bench_function("Create bench tree", |b| b.iter(|| create_bench_tree()));
}

fn compare_from_id_tree(c: &mut Criterion) {
    let louds = Fun::new("LOUDS from IDTree", |b, i| {
        b.iter(|| LOUDSTree::from_id_tree(create_bench_tree()))
    });
    let bp = Fun::new("BP from IDTree", |b, i| {
        b.iter(|| BPTree::from_id_tree(create_bench_tree()))
    });
    c.bench_functions("Create from IDTree", vec![louds, bp], 0);
}

fn compare_is_leaf(c: &mut Criterion) {
    let louds = create_bench_louds();
    let bp = create_bench_bp();
    let louds_fun = Fun::new("LOUDS", move |b, _| b.iter(|| louds.is_leaf(1)));
    let bp_fun = Fun::new("BP", move |b, _| b.iter(|| bp.is_leaf(1)));
    c.bench_functions("Compare is_leaf()", vec![louds_fun, bp_fun], 0);
}

fn compare_parent(c: &mut Criterion) {
    let louds = create_bench_louds();
    let bp = create_bench_bp();
    let louds_fun = Fun::new("LOUDS", move |b, _| b.iter(|| louds.parent(1)));
    let bp_fun = Fun::new("BP", move |b, _| b.iter(|| bp.parent(1)));
    c.bench_functions("Compare parent()", vec![louds_fun, bp_fun], 0);
}

fn compare_first_child(c: &mut Criterion) {
    let louds = create_bench_louds();
    let bp = create_bench_bp();
    let louds_fun = Fun::new("LOUDS", move |b, _| b.iter(|| louds.first_child(1)));
    let bp_fun = Fun::new("BP", move |b, _| b.iter(|| bp.first_child(1)));
    c.bench_functions("Compare first_child()", vec![louds_fun, bp_fun], 0);
}

fn compare_next_sibling(c: &mut Criterion) {
    let louds = create_bench_louds();
    let bp = create_bench_bp();
    let louds_fun = Fun::new("LOUDS", move |b, _| b.iter(|| louds.next_sibling(1)));
    let bp_fun = Fun::new("BP", move |b, _| b.iter(|| bp.next_sibling(1)));
    c.bench_functions("Compare next_sibling()", vec![louds_fun, bp_fun], 0);
}

criterion_group!(
    benches,
    create_bench_idtree,
    compare_load_huge_tree,
    compare_parent,
    compare_from_id_tree,
    compare_is_leaf,
    compare_first_child,
    compare_next_sibling
);
criterion_main!(benches);
