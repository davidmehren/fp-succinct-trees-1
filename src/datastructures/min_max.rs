use bincode::{deserialize, serialize};
use bv::{BitVec, Bits};
use common::errors::NodeError;
use std::fmt::Debug;
use std::ops::Deref;
use std::f64;

#[derive(Serialize, Deserialize)]
pub struct MinMax {
    bits_len: u32,
    bits: BitVec<u8>,
    block_size: u32,
    heap: Vec<MinMaxNode>,
}

impl MinMax {
    pub fn new(bits: BitVec<u8>, block_size: u32) -> MinMax {
        let bits_len = bits.len() as u32;

        let mut number_of_blocks = 0;

        if bits_len % block_size != 0 {
            number_of_blocks = bits_len/block_size + 1;
        }
            else {
                number_of_blocks = bits_len/block_size;
            }

        let max_blocks = 2u32.pow((number_of_blocks as f64).log2().ceil() as u32);

        let heap_size = max_blocks * 2 - 1;

        let number_of_nodes = heap_size - max_blocks + number_of_blocks; // TODO vielleicht unnötig

        let mut heap = vec![MinMaxNode::default(); heap_size as usize];

        let mut heap_index = max_blocks as usize; // n/2 +1

        let mut excess = 0;
        let mut min_excess = 0;
        let mut number_min_excess = 0;
        let mut max_excess = 0;

        for bit_index in 0..bits_len {
            // Werte berechnen:
            if bits.get_bit(bit_index as u64) {
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
                heap.get_mut(heap_index).unwrap()
                    .set_values(&excess, &min_excess, &number_min_excess, &max_excess);
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

    pub fn excess (&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn find_close(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn enclose(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Default,Serialize, Deserialize)]
pub struct MinMaxNode {
    excess : i32,
    min_excess : i32,
    number_min_excess : u64,
    max_excess : i32,
}

impl MinMaxNode {
    pub fn set_values(&mut self, excess: &i32, min_excess: &i32, number_min_excess: &u64, max_excess: &i32) {
        self.excess = *excess;
        self.min_excess = *min_excess;
        self.number_min_excess = *number_min_excess;
        self.max_excess = *max_excess;
    }
}