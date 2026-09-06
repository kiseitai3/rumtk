/*
 *     rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 *     This toolkit aims to be reliable, simple, performant, and standards compliant.
 *     Copyright (C) 2026  Luis M. Santos, M.D. <lsantos@medicalmasses.com>
 *     Copyright (C) 2026  MedicalMasses L.L.C. <contact@medicalmasses.com>
 *
 *     This program is free software: you can redistribute it and/or modify
 *     it under the terms of the GNU General Public License as published by
 *     the Free Software Foundation, either version 3 of the License, or
 *     (at your option) any later version.
 *
 *     This program is distributed in the hope that it will be useful,
 *     but WITHOUT ANY WARRANTY; without even the implied warranty of
 *     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *     GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::base::RUMVec;
pub use branches::{likely as cpu_likely_branch, prefetch_read_data, unlikely as cpu_unlikely_branch};
pub use std::simd::prelude::*;
use crate::rumtk_mem_quick_array_init;

pub const CPU_L1_PREFETCH: i32 = 0;
pub const CPU_L2_PREFETCH: i32 = 1;
pub const CPU_L3_PREFETCH: i32 = 2;
pub const CPU_NONTEMPORAL_PREFETCH: i32 = 3;
pub const CPU_L1_CACHE_LINE_SIZE: usize = 64; // Number of bytes in a typical x86_64 CPU L1 cache line.
pub const CPU_L1_CACHE_SIZE: usize = 32 * 1024; // Number of bytes in a typical x86_64 CPU L1 cache per core.
pub const CPU_PAGE_SIZE: usize = 4 * 1024; // Typical CPU page size
pub const CPU_SIMD_64_SIZE: usize = 64;
pub const CPU_SIMD_32_SIZE: usize = 32;
pub const CPU_SIMD_16_SIZE: usize = 16;
pub const CPU_SIMD_8_SIZE: usize = 8;
pub const CPU_SIMD_AVERAGE_ALU_COUNT: usize = 4;
pub const CPU_SEARCH_WINDOW_1024_SIZE: usize = 1024;
pub const CPU_SEARCH_WINDOW_512_SIZE: usize = 512;
pub const CPU_SEARCH_WINDOW_256_SIZE: usize = 256;
pub const CPU_SEARCH_WINDOW_128_SIZE: usize = 128;
pub const CPU_SEARCH_WINDOW_64_SIZE: usize = 64;
pub const CPU_SEARCH_WINDOW_32_SIZE: usize = 32;
pub const CPU_SEARCH_WINDOW_16_SIZE: usize = 16;


#[cfg(feature = "simd")]
pub type u8xN<const SEARCH_WINDOW_SIZE: usize> = Simd<u8, SEARCH_WINDOW_SIZE>;

////////////////////////////////////////CPU CACHE HINTS///////////////////////////////
#[inline]
pub fn cpu_l3_prefetch(data: *const u8) {
    prefetch_read_data::<u8, CPU_L3_PREFETCH>(data);
}

#[inline]
pub fn cpu_l2_prefetch(data: *const u8) {
    prefetch_read_data::<u8, CPU_L2_PREFETCH>(data);
}

#[inline]
pub fn cpu_l1_prefetch(data: *const u8) {
    prefetch_read_data::<u8, CPU_L1_PREFETCH>(data);
}

#[inline(always)]
pub fn cpu_slice_to_array<const SLICE_SIZE: usize>(chunk: &[u8]) -> &[u8; SLICE_SIZE] {
    chunk.try_into().expect("length mismatch")
}

#[inline(always)]
pub fn cpu_slice_to_array_padded<const SLICE_SIZE: usize, const PAD: u8>(chunk: &[u8]) -> [u8; SLICE_SIZE] {
    let mut result = [PAD; SLICE_SIZE];
    result[..chunk.len()].copy_from_slice(chunk);
    result
}

#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_slice_to_simd<const SLICE_SIZE: usize, const PAD: u8>(chunk: &[u8]) -> u8xN<SLICE_SIZE> {
    u8xN::from_array(cpu_slice_to_array_padded::<SLICE_SIZE, PAD>(chunk))
}

#[inline(always)]
pub fn cpu_slice_splat<const SLICE_SIZE: usize>(input: &[u8]) -> [u8; SLICE_SIZE] {
    let mut result = [0u8; SLICE_SIZE];

    for chunk in result.chunks_mut(input.len()) {
        chunk.copy_from_slice(&input[..chunk.len()]);
    }

    result
}

////////////////////////////////////////SIMD MASKS//////////////////////////////////////////
#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_simd_shift_right_n<const LANE_SIZE: usize, const SHIFT: usize, const PAD: u8>(item: &u8xN<LANE_SIZE>) -> u8xN<LANE_SIZE> {
    item.shift_elements_right::<SHIFT>(PAD)
}

#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_simd_masks<const LANE_SIZE: usize>(pattern: &[u8]) -> RUMVec<u8xN<LANE_SIZE>> {
    let mask = u8xN::<LANE_SIZE>::from_array(cpu_slice_splat(pattern));
    let mut masks = RUMVec::<u8xN<LANE_SIZE>>::with_capacity(pattern.len());

    masks.push(mask);

    for i in 1..pattern.len() {
        let shifted = cpu_simd_shift_right_n::<LANE_SIZE, 1, 0>(&mask);
        masks.push(shifted);
    }

    masks
}

////////////////////////////////////////SEARCH FOR NEEDLE IN HAYSTACK///////////////////////////////

#[inline(always)]
pub fn cpu_find_fallback(chunk: &[u8], byte: u8) -> Option<usize> {
    chunk.iter().position(|c| *c==byte)
}

#[cfg(feature = "simd")]
#[inline]
fn cpu_find_simd_avx2_n<const SEARCH_WINDOW_SIZE: usize>(chunk: &[u8], target: u8xN<SEARCH_WINDOW_SIZE>) -> Option<usize> {
    let data_vec = cpu_slice_to_simd::<SEARCH_WINDOW_SIZE, 0>(chunk);
    let mask = data_vec.simd_eq(target);

    if mask.any() {
        let bitmask = mask.to_bitmask();
        let lane_i = bitmask.trailing_zeros() as usize;
        return Some(lane_i);
    }

    None
}

///
/// Use SIMD to find the index of a byte in a byte slice.
///
/// ## Note
///
/// We try saturating the CPU SIMD ports with ops. If we had to do all [CPU_SIMD_AVERAGE_ALU_COUNT]
/// passes to find the needle, we essentially speculatively precomputed the index. Our worst case
/// is if we only had to do one pass to find the needle, but we are relying on out-of-order execution
/// and the cheapness of SIMD to hide the latency. This also serves as a form of prefetching the
/// byte slice into the cache lines, so beware of thrashing it.
///
///
#[cfg(feature = "simd")]
#[inline]
pub fn cpu_find_simd_n<const LANE_SIZE: usize>
(
    chunk: &[u8],
    byte: u8,
) -> Option<usize>
{
    let mask = u8xN::<LANE_SIZE>::splat(byte);
    let mut iter = chunk.chunks(LANE_SIZE);
    let max_iter = iter.len();
    let large_steps = max_iter / CPU_SIMD_AVERAGE_ALU_COUNT; // div here so consider changing to shifts for extra ns perf
    let mut indx = 0;

    // Try saturating the CPU SIMD ports with ops. If we had to do all 4 passes to find the needle,
    // we essentially speculatively precomputed the index. Our worst case is if we only had to do
    // one pass to find the needle but we are relying on out of order execution and the cheapness of
    // SIMD to hide the latency. This also serves as a form of prefetching byte slice.
    for _ in 0..large_steps {
        let mut indices = rumtk_mem_quick_array_init!(Option<usize>, CPU_SIMD_AVERAGE_ALU_COUNT);
        indices[0] = cpu_find_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        indices[1] = cpu_find_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        indices[2] = cpu_find_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        indices[3] = cpu_find_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);

        for i in indices {
            match i {
                Some(idx) => {
                    return Some(indx + idx);
                }
                None => {
                    indx += LANE_SIZE;
                }
            }
        }
    }

    match iter.next() {
        Some(window) => {
            cpu_find_simd_avx2_n::<LANE_SIZE>(window, mask).map(|idx| indx + idx)
        }
        None => Some(indx + LANE_SIZE),
    }
}

#[cfg(feature = "simd")]
#[inline]
pub fn cpu_find(window: &[u8], byte: u8) -> Option<usize> {
    cpu_find_simd_n::<CPU_SIMD_64_SIZE>(
        window,
        byte,
    )
}

#[cfg(not(feature = "simd"))]
#[inline]
pub fn cpu_find(window: &[u8], byte: u8) -> Option<usize> {
    cpu_find_fallback(
        window,
        byte,
    )
}

////////////////////////////////////////SEARCH FOR CONTINUOUS SLICE OF NEEDLES///////////////////////////////
#[inline(always)]
pub fn cpu_continuous_count_fallback(chunk: &[u8], byte: u8) -> Option<usize> {
    let mut count = 0;
    for i in 0..chunk.len() {
        if cpu_unlikely_branch(chunk[i] != byte) {
            return Some(count);
        }
        count += 1;
    }
    Some(count)
}


#[cfg(feature = "simd")]
#[inline]
fn cpu_continuous_count_simd_avx2_n<const SEARCH_WINDOW_SIZE: usize>(chunk: &[u8], target: u8xN<SEARCH_WINDOW_SIZE>) -> usize {
    let data_vec = cpu_slice_to_simd::<SEARCH_WINDOW_SIZE, 0>(chunk);
    let mask = data_vec.simd_eq(target);

    let bitmask = mask.to_bitmask();
    bitmask.trailing_zeros() as usize
}

///
/// Use SIMD to find the index of a byte in a byte slice.
///
/// ## Note
///
/// We try saturating the CPU SIMD ports with ops. If we had to do all [CPU_SIMD_AVERAGE_ALU_COUNT]
/// passes to find the needle, we essentially speculatively precomputed the index. Our worst case
/// is if we only had to do one pass to find the needle, but we are relying on out-of-order execution
/// and the cheapness of SIMD to hide the latency. This also serves as a form of prefetching the
/// byte slice into the cache lines, so beware of thrashing it.
///
///
#[cfg(feature = "simd")]
#[inline]
pub fn cpu_continuous_count_simd_n<const LANE_SIZE: usize>
(
    chunk: &[u8],
    byte: u8,
) -> Option<usize>
{
    let mask = u8xN::<LANE_SIZE>::splat(byte);
    let mut iter = chunk.chunks(LANE_SIZE);
    let max_iter = iter.len();
    let large_steps = max_iter / CPU_SIMD_AVERAGE_ALU_COUNT; // div here so consider changing to shifts for extra ns perf
    let mut indx = 0;

    // Try saturating the CPU SIMD ports with ops. If we had to do all 4 passes to find the needle,
    // we essentially speculatively precomputed the index. Our worst case is if we only had to do
    // one pass to find the needle but we are relying on out of order execution and the cheapness of
    // SIMD to hide the latency. This also serves as a form of prefetching byte slice.
    for _ in 0..large_steps {
        let mut sizes = rumtk_mem_quick_array_init!(usize, CPU_SIMD_AVERAGE_ALU_COUNT);
        sizes[0] = cpu_continuous_count_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        sizes[1] = cpu_continuous_count_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        sizes[2] = cpu_continuous_count_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);
        sizes[3] = cpu_continuous_count_simd_avx2_n::<LANE_SIZE>(iter.next().unwrap_or_default(), mask);

        for s in sizes {
            if s < LANE_SIZE {
                return Some(indx + s);
            }
            indx += LANE_SIZE;
        }
    }

    match iter.next() {
        Some(window) => {
            Some(indx + cpu_continuous_count_simd_avx2_n::<LANE_SIZE>(window, mask))
        },
        None => Some(indx),
    }
}

#[cfg(feature = "simd")]
#[inline]
pub fn cpu_continuous_count(window: &[u8], byte: u8) -> Option<usize> {
    let start = cpu_find(window, byte)?;
    cpu_continuous_count_simd_n::<CPU_SIMD_64_SIZE>(
        &window[start..],
        byte,
    )
}

#[cfg(not(feature = "simd"))]
#[inline]
pub fn cpu_continuous_count(window: &[u8], byte: u8) -> Option<usize> {
    let start = cpu_find(window, byte)?;
    cpu_continuous_count_fallback(
        &window[start..],
        byte,
    )
}

/////////////////////////////Replacement Helpers///////////////////////////////
#[inline(always)]
pub fn cpu_replace_fallback(data: &mut [u8], pattern: u8, replacement: u8) {
    for i in 0..data.len() {
        if data[i] == pattern {
            data[i] = replacement;
        }
    }
}

#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_find_replace_simd_n<const LANE_SIZE: usize>(chunk: &mut [u8], pattern: u8xN<LANE_SIZE>, replacement: u8xN<LANE_SIZE>) {
    let simd_chunk = u8xN::<LANE_SIZE>::from_array(cpu_slice_to_array_padded::<LANE_SIZE, 0>(chunk));
    let bitmask = simd_chunk.simd_eq(pattern);

    if bitmask.any() {
        replacement.store_select(chunk, bitmask);
    }
}

#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_replace_simd_n<const LANE_SIZE: usize>(data: &mut [u8], pattern: u8, replacement: u8) {
    let mask = u8xN::<LANE_SIZE>::splat(pattern);
    let simd_replacement = u8xN::<LANE_SIZE>::splat(replacement);

    for chunk in data.chunks_mut(LANE_SIZE) {
        cpu_find_replace_simd_n::<LANE_SIZE>(chunk, mask, simd_replacement);
    }
}

#[cfg(feature = "simd")]
#[inline(always)]
pub fn cpu_replace_byte(data: &mut [u8], pattern: u8, replacement: u8) {
    cpu_replace_simd_n::<CPU_SIMD_64_SIZE>(data, pattern, replacement)
}

#[cfg(not(feature = "simd"))]
#[inline(always)]
pub fn cpu_replace_byte(data: &mut [u8], pattern: u8, replacement: u8) {
    cpu_replace_fallback(data, pattern, replacement)
}

/////////////////////////////GATHER ALL INDICES OF NEEDLE IN HAYSTACK///////////////////////////////
pub type CPUTokenStackIndex<const LANE_SIZE: usize> = [u32; LANE_SIZE];
pub type CPUTokenRelativeStackInfo<const LANE_SIZE: usize> = (usize, CPUTokenStackIndex<LANE_SIZE>);
pub type CPUTokenIndexCollection = RUMVec<u32>;
pub type CPUTokenIndexSet = (u8, RUMVec<u32>);
pub type CPUTokenSet = (u8, u32);
pub type CPUTokenSetCollection = RUMVec<CPUTokenSet>;

#[inline(always)]
pub fn cpu_collect_fallback(chunk: &[u8], byte: u8, offset: usize) -> CPUTokenIndexCollection {
    let mut results: CPUTokenStackIndex<CPU_SIMD_64_SIZE> = [0; CPU_SIMD_64_SIZE];
    let mut length = 0;

    for i in 0..chunk.len() {
        if chunk[i]==byte {
            let pos = offset + i;
            results[length] = pos as u32;
            length += 1;
        }
    }

    CPUTokenIndexCollection::from(&results[..length])
}

#[cfg(feature = "simd")]
#[inline]
fn cpu_collect_simd_avx2_n<const LANE_SIZE: usize>(data_vec: &u8xN<LANE_SIZE>, target: u8xN<LANE_SIZE>, offset: usize) -> Option<CPUTokenRelativeStackInfo<LANE_SIZE>> {
    let mut results: CPUTokenStackIndex<LANE_SIZE> = [0; LANE_SIZE];
    let mut length = 0;

    let mask = data_vec.simd_eq(target);

    if mask.any() {
        let items = mask.to_array();

        for i in 0..items.len() {
            if items[i] {
                let pos = offset + i;
                results[length] = pos as u32;
                length += 1;
            }
        }

        return Some((length, results));
    }

    None
}

#[cfg(feature = "simd")]
#[inline]
pub fn cpu_collect_simd_n<const LANE_SIZE: usize>
(
    chunk: &[u8],
    byte: u8,
    offset: usize
) -> CPUTokenIndexCollection
{
    let mask = u8xN::<LANE_SIZE>::splat(byte);
    let (prefix, middle, postfix) = chunk.as_simd::<LANE_SIZE>();

    let mut local_offset: usize = offset;
    let data: CPUTokenIndexCollection = cpu_collect_fallback(prefix, byte, local_offset);
    let mut positions: CPUTokenIndexCollection = CPUTokenIndexCollection::from(data);
    local_offset += prefix.len();

    for window in middle.into_iter() {
        match cpu_collect_simd_avx2_n::<LANE_SIZE>(window, mask, local_offset) {
            Some((len, data)) => {
                positions.extend_from_slice(&data[..len]);
            },
            None => {},
        };
        local_offset += LANE_SIZE;
    }

    let data: CPUTokenIndexCollection = cpu_collect_fallback(postfix, byte, local_offset);
    positions.extend_from_slice(&data);

    positions
}

#[cfg(feature = "simd")]
#[inline]
pub fn cpu_collect(window: &[u8], byte: u8, offset: usize) -> CPUTokenIndexSet {
    let indx = cpu_collect_simd_n::<CPU_SIMD_64_SIZE>(
        window,
        byte,
        offset
    );
    (byte, indx)
}

#[cfg(not(feature = "simd"))]
#[inline]
pub fn cpu_collect(window: &[u8], byte: u8, offset: usize) -> CPUTokenIndexSet {
    let indx = cpu_collect_fallback(
        window,
        byte,
        offset
    );
    (byte, indx)
}

#[inline]
pub fn cpu_tokenize<const WINDOW_SIZE: usize>(haystack: &[u8], bytes: &[u8]) -> CPUTokenSetCollection
{
    let mut results = CPUTokenSetCollection::with_capacity(haystack.len() * size_of::<CPUTokenSet>());
    let mut offset = 0;

    for window in haystack.chunks(WINDOW_SIZE) {
        for byte in bytes {
            let (b, indx) = cpu_collect(window, *byte, offset);

            if !indx.is_empty() {
                for tok_indx in indx {
                    results.push((b, tok_indx));
                }
            }
        }
        offset += window.len();
    }

    results.sort_unstable_by(|a,b| a.1.cmp(&b.1));

    results
}

#[inline]
pub fn cpu_tokenize_rev<const WINDOW_SIZE: usize>(haystack: &[u8], bytes: &[u8]) -> CPUTokenSetCollection
{
    let reversed: Vec<u8> = bytes.iter().rev().cloned().collect();
    cpu_tokenize::<WINDOW_SIZE>(haystack, &reversed)
}
