use std::fmt::Debug;
use bv::{BitVec, Bits};
use common::errors::NodeError;

pub struct MinMax {
    bits_len: usize,
    bits: BitVec<u8>,
    block_size: usize,
    heap: Vec<MinMaxNode>,
}

impl MinMax {
    pub fn new(bits: BitVec<u8>, block_size: usize) -> MinMax {
        let bits_len = bits.len() as usize;

        let mut number_of_blocks = 0;

        if bits_len % block_size != 0 {
            number_of_blocks = bits_len/block_size + 1;
        }
        else {
            number_of_blocks = bits_len/block_size;
        }

        let max_blocks = pow(2, ceil(log2(number_of_blocks)));

        let heap_size = max_blocks * 2 - 1;

        let number_of_nodes = heap_size - max_blocks + number_of_blocks; // TODO vielleicht unnötig

        let heap = vec![Sim::default(); heap_size];

        let iter = bits.iter();
        let next = iter.next();

        let mut heap_index = max_blocks -1; // -1: Korrektur für 0 % (block_size -1) == 0

        let mut excess = 0;
        let mut min_excess = 0;
        let mut number_min_excess = 0;
        let mut max_excess = 0;

        for bit_index in 0..bits_len {
            if bit_index % (block_size - 1) == 0 {
                //Werte in Node speichern

                heap_index += 1;
            }
            // Werte berechnen:

        }

        MinMax{
            bits_len,
            bits,
            block_size,
            heap,
        }
    }

    pub fn excess (u64: index) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn find_close(u64: index) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn enclose(u64: index) -> Result<u64, NodeError> {
        unimplemented!();
    }
}

#[derive(Clone)]
pub struct MinMaxNode {
    mut excess : i32,
    mut min_excess : i32,
    mut number_min_excess : u64,
    mut max_excess : i32,
}

impl MinMaxNode {
    fn set_values (&self, i32: excess, i32: min_excess, u64: number_min_excess, i32: max_excess) {
        &self.excess = excess;
        &self.min_excess = min_excess;
        &self.number_min_excess = number_min_excess;
        &self.max_excess = max_excess;
    }
}

impl Debug for MinMaxNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.excess, self.min_excess, self.number_min_excess, self.max_excess)
    }
}