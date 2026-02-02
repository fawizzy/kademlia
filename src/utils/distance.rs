use std::cmp::Ordering;

use crate::utils::constant::ID_BYTES;

pub fn xor_distance(a: [u8; ID_BYTES], b: [u8; ID_BYTES]) -> [u8; ID_BYTES] {
    let mut distance: [u8; ID_BYTES] = [0; ID_BYTES];
    for i in 0..ID_BYTES {
        distance[i] = a[i] ^ b[i]
    }
    distance
}


pub fn compare_distance(a: [u8; ID_BYTES], b: [u8; ID_BYTES]) -> Ordering {
    for i in 0..ID_BYTES {
        if a[i] > b[i] {return Ordering::Greater;}
        if a[i] < b[i] {return Ordering::Less;}
    }

    Ordering::Equal
}