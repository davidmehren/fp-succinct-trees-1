// Copyright 2018 Daniel Rose and Frederik Stehli.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Range-Min-Max data structure based on Cordova and Navarro (2016)

use bv::BitVec;
use common::errors::NodeError;
use std::cmp;
use std::f64;

/// A Range-Min-Max data structure
#[derive(Serialize, Deserialize)]
pub struct MinMax {
    bits_len: u64,
    bits: BitVec<u8>,
    block_size: u64,
    heap: Vec<MinMaxNode>,
}

impl MinMax {
    pub fn new(bits: BitVec<u8>, block_size: u64) -> Self {
        let bits_len = bits.len();

        let number_of_blocks = if bits_len % block_size != 0 {
            bits_len / block_size + 1
        } else {
            bits_len / block_size
        };

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
                    excess -= 1;
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
                heap[heap_index].set_values(&excess, &min_excess, &number_min_excess, &max_excess);
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

        Self {
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
        let block_number = index / self.block_size;
        let position_in_block = index % self.block_size;
        let mut pre_excess = 0;
        let mut block_rank: u64 = 0;
        let mut j = block_number;
        while j > 0 {
            if (j % 2) == 0 {
                j = (j - 1) / 2;
                pre_excess = self.heap[(2 * j + 1) as usize].excess;
            } else {
                j = (j - 1) / 2;
            }
        }
        for k in (block_number * self.block_size)..index {
            if self.bits[k] {
                block_rank += 1;
            }
        }
        Ok(pre_excess as u64 + (2 * block_rank - position_in_block))
    }

    fn fwd_search(&self, index: u64, diff: i64) -> Result<u64, NodeError> {
        let end_of_block = (index / self.block_size) + self.block_size;
        let index_excess = self.excess(index);
        let mut current_excess = 0;
        let mut j = index;
        let mut found = false;
        while !found && j < end_of_block {
            j += 1;
            if self.bits[j] {
                current_excess += 1;
            } else {
                current_excess -= 1;
            }
            if current_excess == diff {
                found = true;
            }
        }
        let mut current_diff = diff - current_excess;
        //bottom up search
        let mut current_node = (self.heap.len() as u64 / 2 + index / self.block_size) as usize;
        let mut top_down_search = false;
        while !top_down_search && current_node != 0 {
            //if current_node is right child go to parent
            if current_node % 2 == 0 {
                current_node = (current_node - 1) / 2;
            } else {
                current_node += 1;
                if current_diff <= self.heap[current_node].max_excess
                    && current_diff >= self.heap[current_node].min_excess
                {
                    top_down_search = true;
                } else {
                    //current_diff is not in the right child range. go to parent.
                    current_diff = current_diff - self.heap[current_node as usize].excess;
                    current_node = (current_node - 1) / 2;
                }
            }
        }
        //top down search
        while !found && top_down_search{
            if current_node <= self.heap.len() / 2 {
                //todo: durchsuche
            } else {
                let left_child = 2 * current_node + 1;
                let right_child = 2 * current_node + 2;
                if current_diff <= self.heap[left_child].max_excess
                    && current_diff >= self.heap[left_child].min_excess
                    {
                        current_node = left_child;
                    } else {
                    current_node = right_child;
                    current_diff = current_diff - self.heap[left_child].excess;
                }
            }
        }

        Ok(3)
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
    excess: i64,
    min_excess: i64,
    number_min_excess: u64,
    max_excess: i64,
}

impl MinMaxNode {
    pub fn set_values(
        &mut self,
        excess: &i64,
        min_excess: &i64,
        number_min_excess: &u64,
        max_excess: &i64,
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
//            true, true, true, false, true, false, true, true, false, false, false, true, false,
//            true, true, true, false, true, false, false, false, false
//        ];
//        let min_max = MinMax::new(bits, 4);
//        assert_eq!(min_max.enclose(4).unwrap(), 1);
//        assert_eq!(min_max.enclose(6).unwrap(), 1);
//    }
//
//}
