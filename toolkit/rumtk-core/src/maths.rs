use std::ops::{Range};

const base_ten: u8= 10;
pub fn generate_tenth_factor(tenth_place: u32) -> u32 {
    let mut factor: u32 = 1;
    let irange = Range{start: 1, end: tenth_place};
    for i in irange {
        factor *= base_ten as u32;
    }
    return factor;
}