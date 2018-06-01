use std::fmt::Debug;
use bv::{BitVec, Bits};

pub struct MinMax {
    n: usize,
    bits: BitVec<u8>,
    blocksize: usize,
    heap: Vec<MinMaxNode>,
}

impl MinMax {
    pub fn new(bits: BitVec<u8>, blocksize: usize) -> MinMax {
        let n = bits.len() as usize;

        let mut number_of_blocks = 0;

        if (n % blocksize != 0) {
            number_of_blocks = n/blocksize + 1;
        }
        else {
            number_of_blocks = n/blocksize;
        }

        let max_blocks = pow(2, ceil(log2(number_of_blocks)));

        let heap_size = max_blocks * 2 - 1;

        let number_of_nodes = heap_size - max_blocks + number_of_blocks;

        let heap = vec![Sim::default(); heap_size];

        MinMax{
            n,
            bits,
            blocksize,
            heap,
        }

    }
}

#[derive(Clone)]
pub struct MinMaxNode {
    mut excess : i32,
    mut min_excess : i32,
    mut number_min_excess : u64,
    mut max_excess : i32,
}

impl Debug for MinMaxNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {}, {}, {})", self.e, self.m, self.n, self.M, self.starts, self.ends)
    }
}