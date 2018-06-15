// Copyright 2018 Daniel Rose and Frederik Stehli.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Range-Min-Max data structure based on Cordova and Navarro (2016)

use bincode::{deserialize, serialize};
use bv::{BitVec, Bits};
use common::errors::NodeError;
use std::cmp;
use std::f64;
use std::fmt::Debug;
use std::ops::Deref;

/// A Range-Min-Max data structure
#[derive(Serialize, Deserialize)]
pub struct MinMax {
    bits_len: u64,
    bits: BitVec<u8>,
    block_size: u64,
    heap: Vec<MinMaxNode>,
}

impl MinMax {
    pub fn new(bits: BitVec<u8>, block_size: u64) -> MinMax {
        let bits_len = bits.len();

        let mut number_of_blocks = 0;

        if bits_len % block_size != 0 {
            number_of_blocks = bits_len / block_size + 1;
        } else {
            number_of_blocks = bits_len / block_size;
        }

        let max_blocks = 2u64.pow((number_of_blocks as f64).log2().ceil() as u32);

        let heap_size = max_blocks * 2 - 1;

        let number_of_nodes = heap_size - max_blocks + number_of_blocks; // TODO vielleicht unnötig

        let mut heap = vec![MinMaxNode::default(); heap_size as usize];

        let mut heap_index = (max_blocks - 1) as usize; // n/2 +1

        let mut excess = 0;
        let mut min_excess = 0;
        let mut number_min_excess: u64 = 0;
        let mut max_excess = 0;

        for bit_index in 0..bits_len {
            if number_min_excess == 0 {
                //check if this is a new block
                if bits[bit_index] {
                    //initialize the values for the first bit of a block
                    excess = 1;
                    min_excess = 1;
                    number_min_excess = 1;
                    max_excess = 1;
                } else {
                    excess = -1;
                    min_excess = -1;
                    number_min_excess = 1;
                    max_excess = -1;
                }
            } else {
                if !bits[bit_index] {
                    //change the excess depending on the bit
                    excess = excess - 1;
                    if excess == min_excess {
                        number_min_excess += 1;
                    } else if excess < min_excess {
                        min_excess = excess;
                        number_min_excess = 1;
                    }
                } else {
                    excess += 1;
                    if excess > max_excess {
                        max_excess = excess;
                    }
                }
            }
            if (bit_index + 1) % block_size == 0 {
                //check if it is the end of a block
                //save values as Node in a heap
                heap.get_mut(heap_index).unwrap().set_values(
                    &excess,
                    &min_excess,
                    &number_min_excess,
                    &max_excess,
                );
                heap_index += 1;
                //set values beack to zero
                excess = 0;
                min_excess = 0;
                number_min_excess = 0;
                max_excess = 0;
            }
        }

        if heap_size != 1 {
            for index in (max_blocks - 2)..=0 {
                let left_child = &heap[(2 * index + 1) as usize];
                let right_child = &heap[(2 * index + 2) as usize];
                excess = left_child.excess + right_child.excess;
                min_excess = cmp::min(
                    left_child.excess + right_child.min_excess,
                    left_child.min_excess,
                );
                if left_child.excess + right_child.min_excess == left_child.min_excess {
                    number_min_excess =
                        left_child.number_min_excess + right_child.number_min_excess;
                } else if left_child.excess + right_child.min_excess > left_child.min_excess {
                    number_min_excess = right_child.number_min_excess;
                } else {
                    number_min_excess = left_child.number_min_excess;
                }
                max_excess = cmp::max(
                    left_child.excess + right_child.min_excess,
                    left_child.min_excess,
                );
            }
        }

        MinMax {
            bits_len,
            bits,
            block_size,
            heap,
        }
    }

    fn parent(index: u64) -> u64 {
        (index - 1) / 2
    }

    fn left_child(index: u64) -> u64 {
        2 * index + 1
    }

    fn right_child(index: u64) -> u64 {
        2 * index + 2
    }

    pub fn excess(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn find_close(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }

    pub fn enclose(&self, index: u64) -> Result<u64, NodeError> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MinMaxNode {
    excess: i32,
    min_excess: i32,
    number_min_excess: u64,
    max_excess: i32,
}

impl MinMaxNode {
    pub fn set_values(
        &mut self,
        excess: &i32,
        min_excess: &i32,
        number_min_excess: &u64,
        max_excess: &i32,
    ) {
        self.excess = *excess;
        self.min_excess = *min_excess;
        self.number_min_excess = *number_min_excess;
        self.max_excess = *max_excess;
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use bv::BitVec;
//    use bv::Bits;
//
//    #[test]
//    fn test_min_max() {
//        let bits = bit_vec![
//            true, true, true, false, true, false, true, true, false, false, false, true, false,
//            true, true, true, false, true, false, false, false, false
//        ];
//        let min_max = MinMax::new(bits, 4);
//        assert_eq!(min_max.excess(21).unwrap(), 0);
//        assert_eq!(min_max.excess(7).unwrap(), 4);
//        // TODO: Werden schon ungültige index-werte zurückgewiesen?
//    }
//
//    #[test]
//    fn test_excess() {
//        let bits = bit_vec![true, false];
//        let min_max = MinMax::new(bits, 2);
//        assert_eq!(min_max.excess(0).unwrap(), 1);
//        assert_eq!(min_max.excess(1).unwrap(), 0);
//    }
//
//    #[test]
//    fn test_find_close() {
//        let bits = bit_vec![true, true, false, false];
//        let min_max = MinMax::new(bits, 2);
//        assert_eq!(min_max.find_close(0).unwrap(), 3);
//        assert_eq!(min_max.find_close(1).unwrap(), 2);
//    }
//
//    #[test]
//    fn test_enclose() {
//        let bits = bit_vec![
//            true, true, true, false, true, false, true, true, false, false, false, true, false,ccccccfgbjiebkrnlufdricijilvgudgnenievucdlnn

//            true, true, true, false, true, false, false, false, false
//        ];
//        let min_max = MinMax::new(bits, 4);
//        assert_eq!(min_max.enclose(4).unwrap(), 1);
//        assert_eq!(min_max.enclose(6).unwrap(), 1);
//    }
//
//}
