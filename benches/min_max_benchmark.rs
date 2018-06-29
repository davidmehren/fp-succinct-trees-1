#[macro_use]
extern crate criterion;
#[macro_use]
extern crate bv;
extern crate fp_succinct_trees_1;

use criterion::Criterion;

use bv::BitVec;
use bv::Bits;
use fp_succinct_trees_1::datastructures::min_max::MinMax;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("gen_min_max", |b| b.iter(|| {
        let bits =
            bit_vec![true, true, true, false, true, false, false, true, true, false, false, false];
        let min_max = MinMax::new(bits, 4);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);