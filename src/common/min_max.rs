// Copyright 2018 Daniel Rose and Frederik Stehli.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Range-Min-Max data structure based on Cordova and Navarro (2016)
//!
//! Example
//!
//! ```
//! #[macro_use]
//! extern crate bv;
//! # extern crate fp_succinct_trees_1;
//!
//! # fn main() {
//! use bv::BitVec;
//! use bv::Bits;
//! use fp_succinct_trees_1::common::min_max::MinMax;
//!
//! let bits = bit_vec![true, false];
//!        let min_max = MinMax::new(bits, 2);
//!        assert_eq!(min_max.excess(0).unwrap(), 1);
//! # }
//! ```

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

        let number_of_nodes = heap_size - max_blocks + number_of_blocks; // TODO probably unnecessary

        let mut heap = vec![MinMaxNode::default(); heap_size as usize];

        let mut heap_index = (max_blocks - 1) as usize; // n/2 +1

        let mut excess = 0;
        let mut min_excess = 0;
        let mut number_min_excess: u64 = 0;
        let mut max_excess = 0;
        let mut bits_for_block = 0;
        let mut begin_of_block = 0;

        for bit_index in 0..bits_len {
            //check if this is a new block:
            if number_min_excess == 0 {
                begin_of_block = bit_index;
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
            //check if it is the end of a block:
            if (bit_index + 1) % block_size == 0 || bit_index + 1 == bits_len {
                //save values as Node in a heap
                bits_for_block = bit_index - begin_of_block + 1;
                heap[heap_index].set_values(
                    &excess,
                    &min_excess,
                    &number_min_excess,
                    &max_excess,
                    &bits_for_block,
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
            for rev in 0..(heap_size / 2) as u64 {
                //want to iterate reverted
                let index = (heap_size / 2 - 1 - rev) as usize;
                //let left_child = &heap[(2 * index + 1)];
                //let right_child = &heap[(2 * index + 2)];
                let left_child = (2 * index + 1) as usize;
                let right_child = (2 * index + 2) as usize;
                if heap[right_child].number_min_excess > 0 {
                    excess = heap[left_child].excess + heap[right_child].excess;
                    min_excess = cmp::min(
                        heap[left_child].excess + heap[right_child].min_excess,
                        heap[left_child].min_excess,
                    );
                    if heap[left_child].excess + heap[right_child].min_excess
                        == heap[left_child].min_excess
                    {
                        // if the minimal excesses are equal
                        number_min_excess = heap[left_child].number_min_excess
                            + heap[right_child].number_min_excess;
                    } else if heap[left_child].excess + heap[right_child].min_excess
                        < heap[left_child].min_excess
                    {
                        //if the right min excess is greater
                        number_min_excess = heap[right_child].number_min_excess;
                    } else {
                        //if the left min excess is greater
                        number_min_excess = heap[left_child].number_min_excess;
                    }
                    max_excess = cmp::max(
                        heap[left_child].excess + heap[right_child].max_excess,
                        heap[left_child].max_excess,
                    );
                    bits_for_block =
                        heap[left_child].bits_for_node + heap[right_child].bits_for_node;
                    //fill the node
                    heap[index].set_values(
                        &excess,
                        &min_excess,
                        &number_min_excess,
                        &max_excess,
                        &bits_for_block,
                    );
                } else {
                    let excess = heap[left_child].excess;
                    let min_excess = heap[left_child].min_excess;
                    let number_min_excess = heap[left_child].number_min_excess;
                    let max_excess = heap[left_child].max_excess;
                    bits_for_block = heap[left_child].bits_for_node;
                    heap[index].set_values(
                        &excess,
                        &min_excess,
                        &number_min_excess,
                        &max_excess,
                        &bits_for_block,
                    );
                }
            }
        }

        Self {
            bits_len,
            bits,
            block_size,
            heap,
        }
    }

    fn parent(&self, index: usize) -> usize {
        (index - 1) / 2
    }

    fn left_child(&self, index: usize) -> usize {
        2 * index + 1
    }

    fn right_child(&self, index: usize) -> usize {
        2 * index + 2
    }

    fn is_leaf(&self, index: usize) -> bool {
        if index >= self.heap.len() / 2 {
            return true;
        }
        false
    }

    pub fn excess(&self, index: u64) -> Result<u64, NodeError> {
        if index >= self.bits.len() {
            return Err(NodeError::NotANodeError);
        }
        let block_number = (index / self.block_size);
        let position_in_block = index % self.block_size;
        let mut pre_excess: i64 = 0;
        let mut block_excess: i64 = 0;
        let mut heap_number = block_number + (self.heap.len() as u64 / 2);
        while heap_number > 0 {
            if (heap_number % 2) == 0 {
                heap_number = (heap_number - 1) / 2;
                pre_excess += self.heap[(2 * heap_number + 1) as usize].excess;
            } else {
                heap_number = (heap_number - 1) / 2;
            }
        }
        for k in (block_number * self.block_size)..=index {
            if self.bits[k] {
                block_excess += 1;
            } else {
                block_excess -= 1;
            }
        }
        Ok((pre_excess + block_excess) as u64)
    }

    fn fwd_search(&self, index: u64, diff: i64) -> Result<u64, NodeError> {
        let end_of_block = (index / self.block_size) * self.block_size + self.block_size;
        let index_excess = self.excess(index);
        let mut current_excess = 0;
        let mut position_in_block = index;

        let mut found = false;
        let mut bottom_up_search = false;
        let mut top_down_search = false;
        let mut block_search = false;
        while !found && position_in_block < end_of_block - 1 {
            position_in_block += 1;
            if self.bits[position_in_block] {
                current_excess += 1;
            } else {
                current_excess -= 1;
            }
            if current_excess == diff - 1 {
                found = true;
            }
        }
        let mut current_diff = diff - 1 - current_excess;
        bottom_up_search = true;
        if (!found) {
            //bottom up search
            let mut current_node = (self.heap.len() as u64 / 2 + index / self.block_size) as usize;
            while bottom_up_search && current_node != 0 {
                //if current_node is right child go to parent
                if current_node % 2 == 0 {
                    current_node = (current_node - 1) / 2;
                } else {
                    current_node += 1;
                    if current_diff <= self.heap[current_node].max_excess
                        && current_diff >= self.heap[current_node].min_excess
                    {
                        bottom_up_search = false;
                        top_down_search = true;
                    } else {
                        //current_diff is not in the right child range. go to parent.
                        current_diff = current_diff - self.heap[current_node as usize].excess;
                        current_node = (current_node - 1) / 2;
                    }
                }
            }
            //top down search
            while top_down_search {
                if current_node >= self.heap.len() / 2 {
                    top_down_search = false;
                    block_search = true;
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
            position_in_block = (current_node - self.heap.len() / 2) as u64 * self.block_size;
            let block_start = position_in_block;
            let end_of_target_block = block_start + self.block_size;
            while !found && block_search && position_in_block < end_of_target_block {
                if self.bits[position_in_block] {
                    // - and + inverted!
                    current_diff -= 1;
                } else {
                    current_diff += 1;
                }
                if current_diff == 0 {
                    found = true;
                } else {
                    position_in_block += 1;
                }
            }
        }
        Ok(position_in_block)
    }

    fn bwd_search(&self, index: u64, diff: i64) -> Result<u64, NodeError> {
        let mut block_no = index / self.block_size;
        let mut begin_of_block = block_no * self.block_size;
        let mut end_of_block = begin_of_block + self.block_size - 1;
        let mut current_node = (self.heap.len() / 2) as u64 + block_no;

        let index_excess = self.excess(index).unwrap() as i64;
        let mut current_excess = index_excess as i64;

        let mut position = index;
        let mut found = false;

        while !found && position > begin_of_block {
            if self.bits[position] {
                current_excess -= 1;
            } else {
                current_excess += 1;
            }
            position -= 1;
            if current_excess == index_excess + diff {
                found = true;
            }
        }

        if !found {
            let mut look_for = diff + index_excess - current_excess;
            let mut bottom_up = true;
            let mut top_down = false;
            let mut block_search = false;

            while bottom_up && current_node > 0 {
                if current_node % 2 == 0 {
                    if self.heap[current_node as usize - 1].max_excess >= -1 * look_for
                        && self.heap[current_node as usize - 1].min_excess <= -1 * look_for
                    {
                        bottom_up = false;
                        top_down = true;
                        current_node -= 1;
                    } else {
                        look_for = look_for + self.heap[current_node as usize - 1].excess;
                        current_node = (current_node - 1) / 2;
                    }
                } else {
                    current_node = (current_node - 1) / 2;
                }
            }

            while top_down {
                if current_node >= self.heap.len() as u64 / 2 {
                    top_down = false;
                    block_search = true;
                } else {
                    if self.heap[current_node as usize * 2 + 2].max_excess
                        - self.heap[current_node as usize * 2 + 2].min_excess
                        >= look_for.abs()
                    {
                        current_node = current_node * 2 + 2;
                    } else if self.heap[current_node as usize * 2 + 1].max_excess
                        - self.heap[current_node as usize * 2 + 1].min_excess
                        >= look_for.abs()
                    {
                        current_node = current_node * 2 + 1;
                    } else {
                        //todo konnte nicht gefunden werden!!
                    }
                }
            }

            /*while top_down {
                if current_node <= self.heap.len() as u64 / 2 {
                    top_down = false;
                    block_search = true;
                } else {
                    if self.heap[current_node as usize * 2 + 2].max_excess >= -1 * look_for
                        && self.heap[current_node as usize * 2 + 2].min_excess <= -1 * look_for
                        {
                            current_node = current_node * 2 + 2;
                        } else if self.heap[current_node as usize * 2 + 1].max_excess >= -1 * look_for
                        && self.heap[current_node as usize * 2 + 1].min_excess <= -1 * look_for
                        {
                            current_node = current_node * 2 + 1;
                        } else {
                        //todo konnte nicht gefunden werden!!
                    }
                }
            }*/

            if block_search {
                block_no = current_node - (self.heap.len() / 2) as u64;
                begin_of_block = block_no * self.block_size;
                end_of_block = begin_of_block + self.block_size - 1;
                position = end_of_block;
                while !found && position >= begin_of_block {
                    if self.bits[position + 1] {
                        look_for += 1;
                    } else {
                        look_for -= 1;
                    }
                    if look_for == 0 {
                        found = true;
                    } else {
                        position -= 1
                    }
                }
            }
        }
        if found {
            Ok(position)
        } else {
            //todo konnte nicht gefunden werden!!
            Ok(10000000)
        }
    }

    pub fn find_close(&self, index: u64) -> Result<u64, NodeError> {
        self.fwd_search(index, 0)
    }

    pub fn enclose(&self, index: u64) -> Result<u64, NodeError> {
        self.bwd_search(index, 1)
    }

    pub fn rank_1(&self, index: u64) -> Result<u64, NodeError> {
        if index >= self.bits.len() {
            Err(NodeError::NotANodeError)
        } else {
            let block_no = (index / self.block_size);
            let begin_of_block = block_no * self.block_size;
            let mut rank = 0;

            // Count 1s in the last block
            for k in begin_of_block..=index {
                if self.bits[k] {
                    rank += 1;
                }
            }

            // TODO: rewrite to use helper functions
            let mut current_node = ((self.heap.len() / 2) as u64 + block_no) as usize;

            while current_node > 0 {
                let old_node = current_node;
                current_node = self.parent(current_node);
                if self.left_child(current_node) != old_node {
                    // (excess of node + number of bits for node)/2 = number of 1-bits for node
                    rank += self.ones_for_node(self.left_child(current_node));
                }
            }

            Ok(rank as u64)
        }
    }

    pub fn rank_0(&self, index: u64) -> Result<u64, NodeError> {
        let result = (index - self.rank_1(index).unwrap()) as i64;
        if result < 0 {
            return Err(NodeError::NotANodeError);
        }
        Ok(index - self.rank_1(index).unwrap() + 1)
    }

    pub fn select_1(&self, rank: u64) -> Result<u64, NodeError> {
        if rank > (self.bits_len / 2) as u64 {
            // case: no "1" with given rank exists
            return Err(NodeError::NotANodeError);
        }
        Ok(self.select_1_recursive(rank as i64, 0) as u64)
    }

    fn select_1_recursive(&self, rank: i64, heap_index: usize) -> i64 {
        if self.is_leaf(heap_index) {
            // recursion termination: return index of kth "1" in block for k = rank
            let block_no = (heap_index - self.heap.len() / 2) as i64;
            let begin_of_block = block_no * self.block_size as i64;
            let end_of_block = begin_of_block + self.block_size as i64;
            let mut remaining_rank = rank;
            let mut index = begin_of_block;
            // for-loop ends at begin_of_block + bits_for_node because last block might be underfull
            for bits_index in
                begin_of_block..begin_of_block + self.heap[heap_index].bits_for_node as i64
            {
                if self.bits[bits_index as u64] && remaining_rank > 0 {
                    remaining_rank -= 1;
                    index = bits_index;
                }
            }
            return index;
        } else {
            let no_of_ones = self.ones_for_node(self.left_child(heap_index));
            if no_of_ones >= rank {
                // case: the sought index belongs to left child: recursive call for lc with rank
                return self.select_1_recursive(rank, self.left_child(heap_index));
            } else {
                // case: the sought index belongs to right child: recursive call
                // for rc with rank - 1s belonging to left child
                self.select_1_recursive(rank as i64 - no_of_ones, self.right_child(heap_index))
            }
        }
    }

    pub fn select_0(&self, rank: u64) -> Result<u64, NodeError> {
        if rank > (self.bits_len / 2) as u64 {
            // case: no "0" with given rank exists
            return Err(NodeError::NotANodeError);
        }
        Ok(self.select_0_recursive(rank as i64, 0) as u64)
    }

    fn select_0_recursive(&self, rank: i64, heap_index: usize) -> i64 {
        if self.is_leaf(heap_index) {
            // recursion termination: return index of kth "0" in block for k = rank
            let block_no = (heap_index - self.heap.len() / 2) as i64;
            let begin_of_block = block_no * self.block_size as i64;
            let end_of_block = begin_of_block + self.block_size as i64;
            let mut remaining_rank = rank;
            let mut index = begin_of_block;
            // for-loop ends at begin_of_block + bits_for_node because last block might be underfull
            for bits_index in
                begin_of_block..begin_of_block + self.heap[heap_index].bits_for_node as i64
            {
                if !self.bits[bits_index as u64] && remaining_rank > 0 {
                    remaining_rank -= 1;
                    index = bits_index;
                }
            }
            return index;
        } else {
            let no_of_zeroes = self.heap[self.left_child(heap_index)].bits_for_node as i64
                - self.ones_for_node(self.left_child(heap_index));
            if no_of_zeroes >= rank {
                // case: the sought index belongs to left child: recursive call for lc with rank
                return self.select_0_recursive(rank, self.left_child(heap_index));
            } else {
                // case: the sought index belongs to right child: recursive call
                // for rc with rank - 0s belonging to left child
                self.select_0_recursive(rank as i64 - no_of_zeroes, self.right_child(heap_index))
            }
        }
    }

    /// Returns the number of 1s belonging to the heap node
    fn ones_for_node(&self, heap_index: usize) -> i64 {
        ((self.heap[heap_index].bits_for_node as i64 + self.heap[heap_index].excess) / 2)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MinMaxNode {
    excess: i64,
    min_excess: i64,
    number_min_excess: u64,
    max_excess: i64,
    bits_for_node: u64,
}

impl MinMaxNode {
    pub fn set_values(
        &mut self,
        excess: &i64,
        min_excess: &i64,
        number_min_excess: &u64,
        max_excess: &i64,
        no_of_bits_for_node: &u64,
    ) {
        self.excess = *excess;
        self.min_excess = *min_excess;
        self.number_min_excess = *number_min_excess;
        self.max_excess = *max_excess;
        self.bits_for_node = *no_of_bits_for_node;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bv::BitVec;
    use bv::Bits;

    #[test]
    fn test_min_max_construction() {
        let bits =
            bit_vec![true, true, true, false, true, false, false, true, true, false, false, false];
        let min_max = MinMax::new(bits, 4);
        //heap has the correct length
        assert_eq!(min_max.heap.len(), 7);
        //the blocks contents are correct
        assert_eq!(min_max.heap[3].excess, 2);
        assert_eq!(min_max.heap[3].min_excess, 1);
        assert_eq!(min_max.heap[3].number_min_excess, 1);
        assert_eq!(min_max.heap[3].max_excess, 3);

        assert_eq!(min_max.heap[4].excess, 0);
        assert_eq!(min_max.heap[4].min_excess, -1);
        assert_eq!(min_max.heap[4].number_min_excess, 1);
        assert_eq!(min_max.heap[4].max_excess, 1);

        assert_eq!(min_max.heap[5].excess, -2);
        assert_eq!(min_max.heap[5].min_excess, -2);
        assert_eq!(min_max.heap[5].number_min_excess, 1);
        assert_eq!(min_max.heap[5].max_excess, 1);

        assert_eq!(min_max.heap[6].excess, 0);
        assert_eq!(min_max.heap[6].min_excess, 0);
        assert_eq!(min_max.heap[6].number_min_excess, 0);
        assert_eq!(min_max.heap[6].max_excess, 0);
        //right subtree has the correct content
        assert_eq!(min_max.heap[2].excess, -2);
        assert_eq!(min_max.heap[2].min_excess, -2);
        assert_eq!(min_max.heap[2].number_min_excess, 1);
        assert_eq!(min_max.heap[2].max_excess, 1);
        //left subtree has the correct content
        assert_eq!(min_max.heap[1].excess, 2);
        assert_eq!(min_max.heap[1].min_excess, 1);
        assert_eq!(min_max.heap[1].number_min_excess, 2);
        assert_eq!(min_max.heap[1].max_excess, 3);
        //root node has the correct content
        assert_eq!(min_max.heap[0].excess, 0);
        assert_eq!(min_max.heap[0].min_excess, 0);
        assert_eq!(min_max.heap[0].number_min_excess, 1);
        assert_eq!(min_max.heap[0].max_excess, 3);
    }

    #[test]
    fn test_min_max_construction2() {
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.heap[0].excess, 0);
        assert_eq!(min_max.heap[1].excess, 4);
        assert_eq!(min_max.heap[2].excess, -4);
        assert_eq!(min_max.heap[3].excess, 4);
        assert_eq!(min_max.heap[4].excess, 0);
        assert_eq!(min_max.heap[5].excess, -4);
        assert_eq!(min_max.heap[6].excess, 0);
        //
        assert_eq!(min_max.heap[7].excess, 2);
        assert_eq!(min_max.heap[8].excess, 2);
        assert_eq!(min_max.heap[9].excess, -2);
        assert_eq!(min_max.heap[10].excess, 2);
        assert_eq!(min_max.heap[11].excess, -2);
        assert_eq!(min_max.heap[12].excess, -2);
        assert_eq!(min_max.heap[13].excess, 0);
        assert_eq!(min_max.heap[14].excess, 0);
    }

    #[test]
    fn test_min_max() {
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.excess(21).unwrap(), 0);
        assert_eq!(min_max.excess(7).unwrap(), 4);
    }

    #[test]
    fn test_excess() {
        let bits = bit_vec![true, false];
        let min_max = MinMax::new(bits, 2);
        assert_eq!(min_max.excess(0).unwrap(), 1);
        assert_eq!(min_max.excess(1).unwrap(), 0);
        assert_eq!(min_max.excess(2).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn test_fwd_search() {
        let bits =
            bit_vec![true, true, true, false, true, false, false, true, true, false, false, false];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.fwd_search(0, 0).unwrap(), 11);
        assert_eq!(min_max.fwd_search(1, 0).unwrap(), 6);
        assert_eq!(min_max.fwd_search(5, 2).unwrap(), 8);
        assert_eq!(min_max.fwd_search(4, 0).unwrap(), 5);
        assert_eq!(min_max.fwd_search(4, 1).unwrap(), 8);
        assert_eq!(min_max.fwd_search(0, 3).unwrap(), 2);
    }

    #[test]
    fn test_find_close() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 2);
        assert_eq!(min_max.find_close(0).unwrap(), 3);
        assert_eq!(min_max.find_close(1).unwrap(), 2);
    }

    #[test]
    fn test_bwd_search() {
        let bits =
            bit_vec![true, true, true, false, true, false, false, true, true, false, false, false];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.bwd_search(7, 1).unwrap(), 4);
        assert_eq!(min_max.bwd_search(5, -1).unwrap(), 0);
        assert_eq!(min_max.bwd_search(10, 0).unwrap(), 6);
    }

    #[test]
    #[ignore]
    fn test_enclose() {
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.enclose(4).unwrap(), 1);
        assert_eq!(min_max.enclose(6).unwrap(), 1);
    }

    #[test]
    fn test_rank_1() {
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.rank_1(11).unwrap(), 7);
        assert_eq!(min_max.rank_1(21).unwrap(), 11);
    }

    #[test]
    fn test_rank_0() {
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.rank_0(12).unwrap(), 6);
        assert_eq!(min_max.rank_0(17).unwrap(), 7);
        assert_eq!(min_max.rank_0(21).unwrap(), 11);
    }

    #[test]
    fn test_parent() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.parent(2), 0);
    }

    #[test]
    fn test_left_child() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.left_child(0), 1);
    }

    #[test]
    fn test_right_child() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.right_child(0), 2);
    }

    #[test]
    fn test_is_leaf() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 1);
        assert_eq!(min_max.is_leaf(0), false);
        // first leaf node:
        assert_eq!(min_max.is_leaf(3), true);
    }

    #[test]
    fn test_ones_for_node() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 1);
        assert_eq!(min_max.ones_for_node(0), 2);
        assert_eq!(min_max.ones_for_node(1), 2);
        assert_eq!(min_max.ones_for_node(6), 0);
    }

    #[test]
    fn test_select_1() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 1);
        assert_eq!(min_max.select_1(2).unwrap(), 1);
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.select_1(4).unwrap(), 4);
        assert_eq!(min_max.select_1(10).unwrap(), 15);
        assert_eq!(min_max.select_1(11).unwrap(), 17);
        assert_eq!(min_max.select_1(12).unwrap_err(), NodeError::NotANodeError);
    }

    #[test]
    fn test_select_0() {
        let bits = bit_vec![true, true, false, false];
        let min_max = MinMax::new(bits, 1);
        assert_eq!(min_max.select_0(2).unwrap(), 3);
        let bits = bit_vec![
            true, true, true, false, true, false, true, true, false, false, false, true, false,
            true, true, true, false, true, false, false, false, false
        ];
        let min_max = MinMax::new(bits, 4);
        assert_eq!(min_max.select_0(1).unwrap(), 3);
        assert_eq!(min_max.select_0(6).unwrap(), 12);
        assert_eq!(min_max.select_0(11).unwrap(), 21);
        assert_eq!(min_max.select_0(12).unwrap_err(), NodeError::NotANodeError);
    }

}
