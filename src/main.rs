extern crate fp_succinct_trees_1;
extern crate failure;

use fp_succinct_trees_1::trees::bp_tree::BPTree;

use failure::Error;

fn main() {
    if let Err(ref e) = run() {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let _ = writeln!(stderr, "Error: {}", e);
        for cause in e.causes() {
            let _ = writeln!(stderr, "Caused by: {}", cause);
        }
    }
}

fn run() -> Result<(), Error> {
    BPTree::stub_create();
    return Ok(());
}