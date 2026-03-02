use std::collections::VecDeque;

use crate::dht::node::Node;
use crate::utils::constant::{ID_BYTES, K};

#[derive(Debug, Clone)]
pub struct Bucket {
    pub nodes: VecDeque<Option<Node>>,
    // size: u8
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            nodes: VecDeque::with_capacity(K),
        } //.into to convert type u8 to usize
    }
    pub fn get_bucket_index(distance: [u8; ID_BYTES]) -> usize {
        // Find the position of the most significant bit
        // Start from the most significant byte (index 0)
        for (byte_index, &byte) in distance.iter().enumerate() {
            if byte != 0 {
                let bit_pos = 7 - byte.leading_zeros() as usize;
                // Total bits from the right
                let remaining_bytes = ID_BYTES - 1 - byte_index;
                return remaining_bytes * 8 + bit_pos;
            }
        }
        0
    }
}
