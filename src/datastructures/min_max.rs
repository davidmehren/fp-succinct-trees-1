use std::fmt::Debug;
use bv::{BitVec, Bits};
use common::errors::NodeError;
use std::ops::Deref;

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

        let mut heap = vec![MinMaxNode::default(); heap_size];

        let iter = bits.iter();
        let next = iter.next();

        let mut heap_index = max_blocks; // n/2 +1

        let mut excess = 0;
        let mut min_excess = 0;
        let mut number_min_excess = 0;
        let mut max_excess = 0;

        for bit_index in 0..bits_len {
            // Werte berechnen:
            if bits.get_bit(bit_index) {
                excess += 1;
            }
                else {
                    excess -= 1;
                }
            if excess > max_excess {
                max_excess = excess;
            }
            if bit_index != 0 && (bit_index % block_size) - 1 == 0 {
                //Werte in Node speichern
//                let node = MinMaxNode::Default();
//                node = heap.get_mut(heap_index);
                heap.get_mut(heap_index).set_values(excess, min_excess, number_min_excess, max_excess);
                heap_index += 1;
                excess = 0;
                min_excess = 0;
                number_min_excess = 0;
                max_excess = 0;
            }
        }

        MinMax{
            bits_len,
            bits,
            block_size,
            heap,
        }
    }

    pub fn excess (index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn find_close(index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn enclose(index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Default)]
pub struct MinMaxNode {
    excess : i32,
    min_excess : i32,
    number_min_excess : u64,
    max_excess : i32,
}

impl MinMaxNode {
    pub fn set_values (&self, &excess: i32, &min_excess: i32, &number_min_excess: u64, &max_excess: i32) {
        self.excess = excess;
        self.min_excess = min_excess;
        self.number_min_excess = number_min_excess;
        self.max_excess = max_excess;
    }
}

//impl Debug for MinMaxNode {
//    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//        write!(f, "({}, {}, {}, {})", self.excess, self.min_excess, self.number_min_excess, self.max_excess)
//    }
//}