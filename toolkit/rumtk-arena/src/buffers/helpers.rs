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
use crate::as_slice_mut;
use crate::base::*;
use crate::buffers::RUMBuffer;
use crate::cpu::*;
use rand::{distr::Alphanumeric, RngExt};
use std::alloc::{alloc, Layout};

///
/// Convert slice of `&[u8]` to [RUMBuffer].
///
/// ## Example
/// ```
/// use rumtk_arena::buffers::slice_to_buffer;
/// use rumtk_arena::buffers::*;
///
/// const expected: &str = "Hello World!";
/// let buffer = RUMBuffer::from(expected.as_bytes());
/// let result = slice_to_buffer(expected.as_bytes());
///
/// assert_eq!(result, buffer, "Slice to RUMBuffer conversion failed!");
/// ```
///
#[inline(always)]
pub fn slice_to_buffer(buffer: &[u8]) -> RUMBuffer {
    RUMBuffer::from(buffer)
}

///
/// Generates a new random buffer using the `rand` crate and wrapped inside a [RUMBuffer](RUMBuffer).
///
/// The buffer size can be adjusted via the turbofish method => `new_random_buffer::<10>()`.
///
/// ## Example
///
/// ```
/// use rumtk_arena::buffers::{new_random_buffer, DEFAULT_BUFFER_CHUNK_SIZE};
///
/// let buffer = new_random_buffer::<DEFAULT_BUFFER_CHUNK_SIZE>();
///
/// assert_eq!(buffer.is_empty(), false, "Function returned an empty random buffer which was unexpected!");
/// assert_eq!(buffer.len(), DEFAULT_BUFFER_CHUNK_SIZE, "The new random buffer does not have the expected size!");
/// ```
///
#[inline(always)]
pub fn new_random_buffer<'a, const N: usize>() -> RUMVec<u8> {
    let buffer = unsafe { alloc(Layout::from_size_align_unchecked(N, size_of::<u8>())) };
    let slice = as_slice_mut(buffer, N * size_of::<u8>());

    rand::fill(slice);
    unsafe { RUMVec::from_raw_parts(buffer, N, N) }
}

///
/// Generates a new random buffer using the `rand` crate and wrapped inside a [RUMBuffer](RUMBuffer).
///
/// The buffer size can be adjusted via the turbofish method => `new_random_buffer::<10>()`.
///
/// ## Example
///
/// ```
/// use rumtk_arena::buffers::{new_random_buffer, DEFAULT_BUFFER_CHUNK_SIZE};
///
/// let buffer = new_random_buffer::<DEFAULT_BUFFER_CHUNK_SIZE>();
///
/// assert_eq!(buffer.is_empty(), false, "Function returned an empty random buffer which was unexpected!");
/// assert_eq!(buffer.len(), DEFAULT_BUFFER_CHUNK_SIZE, "The new random buffer does not have the expected size!");
/// ```
///
#[inline(always)]
pub fn new_random_rumbuffer<const N: usize>() -> RUMBuffer {
    slice_to_buffer(&new_random_buffer::<N>())
}

///
/// Generates a new random string using the `rand` crate and wrapped inside a [RUMString](RUMString).
///
/// The buffer size can be adjusted via the turbofish method => `new_random_string_buffer::<10>()`.
///
/// ## Example
///
/// ```
/// use rumtk_arena::buffers::{new_random_string_buffer, DEFAULT_BUFFER_CHUNK_SIZE};
///
/// let buffer = new_random_string_buffer::<DEFAULT_BUFFER_CHUNK_SIZE>();
///
/// assert_eq!(buffer.is_empty(), false, "Function returned an empty random buffer which was unexpected!");
/// assert_eq!(buffer.len(), DEFAULT_BUFFER_CHUNK_SIZE, "The new random buffer does not have the expected size!");
/// ```
///
pub fn new_random_string_buffer<const N: usize>() -> RUMString {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(N) // Length of the string
        .map(char::from)
        .collect()
}

///
/// Generates a new random set of [RUMString] using the `rand` crate.
///
/// The buffer size for each item can be adjusted via the turbofish method => `new_random_string_set::<10>()`.
///
/// ## Example
///
/// ```
/// use rumtk_arena::buffers::{new_random_string_set, DEFAULT_BUFFER_CHUNK_SIZE};
///const item_count: usize = 5;
///
/// let buffer = new_random_string_set::<DEFAULT_BUFFER_CHUNK_SIZE>(item_count);
///
/// assert_eq!(buffer.is_empty(), false, "Function returned an empty random buffer which was unexpected!");
/// assert_eq!(buffer.len(), item_count, "The new random buffer does not have the expected item count!");
/// assert_eq!(buffer.get(0).unwrap().len(), DEFAULT_BUFFER_CHUNK_SIZE, "The new random buffer does not have the expected size!");
/// ```
///
pub fn new_random_string_set<const N: usize>(item_count: usize) -> RUMVec<RUMString> {
    let mut set = RUMVec::<RUMString>::with_capacity(item_count);

    for _ in 0..item_count {
        set.push(new_random_string_buffer::<N>())
    }

    set
}

///
/// Convert buffer to string.
///
/// ## Example
/// ```
/// use rumtk_arena::buffers::buffer_to_string;
/// use rumtk_arena::buffers::*;
///
/// const expected: &str = "Hello World!";
/// let buffer = RUMBuffer::from(expected.as_bytes());
/// let result = buffer_to_string(&buffer).unwrap();
///
/// assert_eq!(result, expected, "Buffer to RUMString conversion failed!");
/// ```
///
#[inline(always)]
pub fn buffer_to_string(buffer: &[u8]) -> RUMResult<RUMString> {
    match RUMString::from_utf8(buffer.to_vec()) {
        Ok(string) => Ok(string),
        Err(e) => Err(rumtk_format!("Failure to parse incoming UTF-8 string: {}", e)),
    }
}

#[inline(always)]
pub fn buffer_to_str(buffer: &[u8]) -> RUMResult<&str> {
    match std::str::from_utf8(buffer) {
        Ok(string) => Ok(string),
        Err(e) => Err(rumtk_format!("Failure to parse incoming UTF-8 string: {}", e)),
    }
}

#[inline(always)]
pub fn buffer_count(buffer: &[u8], pattern: u8) -> usize {
    bytecount::count(buffer, pattern)
}

#[inline(always)]
pub fn buffer_contains(buffer: &[u8], pattern: u8) -> bool {
    buffer_find_byte(buffer, pattern).unwrap_or(buffer.len()) < buffer.len()
}

#[inline(always)]
pub fn buffer_find_byte(buffer: &[u8], byte: u8) -> Option<usize> {
    // Optimize the empty case
    if buffer.is_empty() { return None }

    // Optimize the byte is next door case
    if buffer[0] == byte { return Some(0) }

    // Attempt normal search otherwise.
    cpu_find(buffer, byte)
}

#[inline(always)]
pub fn buffer_find(buffer: &[u8], pattern: &[u8]) -> usize {
    if buffer.is_empty() {
        return buffer.len();
    }

    let start_pattern_byte = pattern[0];
    let pattern_length = pattern.len();
    let buffer_len = buffer.len();
    let end = buffer_len - pattern_length;
    let mut cursor = 0;

    loop {
        match buffer_find_byte(&buffer[cursor..], start_pattern_byte) {
            Some(indx) => {
                cursor += indx;
                if cursor > end { return buffer_len; }

                let slice = &buffer[cursor..cursor + pattern_length];
                if  slice == pattern {
                    return cursor;
                }

                cursor += 1;
            },
            None => return buffer_len,
        }
    }
}

#[inline(always)]
pub fn buffer_find_instances<'a>(buffer: &'a [u8], pattern: &[u8]) -> RUMVec<(usize, &'a [u8])> {
    if buffer.is_empty() {
        return RUMVec::new();
    }

    let pattern_length = pattern.len();
    let buffer_length = buffer.len() - pattern_length;
    let mut instances = RUMVec::<(usize, &[u8])>::with_capacity(100);

    let mut cursor = buffer_find(buffer, pattern);
    let mut cumulative = cursor;
    let mut remainder = &buffer[..];

    while cumulative < buffer_length {
        instances.push((cumulative, &remainder[..cursor]));
        let next = cursor + pattern_length;
        if next <= remainder.len() {
            remainder = &remainder[cursor + pattern_length..];
            cursor = buffer_find(remainder, pattern);
            cumulative += cursor;
        } else {
            cumulative += remainder.len();
        }
    }

    instances
}

#[inline(always)]
pub fn buffer_replace_in_place<'a>(buffer: &'a mut [u8], pattern: &[u8], replacement: &[u8]) {
    // Optimize the nothing passed to us case.
    if buffer.is_empty() || pattern.is_empty() || replacement.is_empty() {
        return;
    }

    // Optimize the single byte replace using SIMD
    if pattern.len() == 1  {
        cpu_replace_byte(buffer, pattern[0], replacement[0]);
        return;
    }

    let replacement_length = replacement.len();
    let mut cursor = buffer_find(&buffer, pattern);
    let mut remainder = buffer;

    while cursor < remainder.len() {
        for i in 0..replacement_length {
            remainder[cursor + i] = replacement[i];
        }

        remainder = &mut remainder[cursor + pattern.len()..];
        cursor = buffer_find(remainder, pattern);
    }
}

#[inline(always)]
pub fn buffer_replace(buffer: &[u8], pattern: &[u8], replacement: &[u8]) -> RUMBuffer {
    match buffer.is_empty() {
        true => RUMBuffer::from(buffer),
        false => {
            let pattern_length = pattern.len();
            let replacement_length = replacement.len();
            let instances = buffer_find_instances(&buffer, pattern);
            let mut new_buffer =  RUMVec::with_capacity(buffer.len() + (instances.len() * (replacement_length)));
            let mut last = 0;

            for (indx, chunk) in instances {
                new_buffer.extend_from_slice(chunk);
                new_buffer.extend_from_slice(replacement);
                last = indx + pattern_length;
            }

            new_buffer.extend_from_slice(&buffer[last..]);
            RUMBuffer::from(new_buffer)
        }
    }
}

#[inline]
pub fn buffer_trim(buffer: &[u8]) -> RUMBuffer {
    let trimmed = buffer_slice_trim(buffer);
    RUMBuffer::from_parts(trimmed.as_ptr(), trimmed.len(), false)
}

#[inline(always)]
pub fn buffer_slice_trim(buffer: &[u8]) -> &[u8] {
    buffer.trim_ascii()
}

pub fn buffer_has_pattern(buffer: &[u8], pattern: &[u8]) -> bool {
    buffer_find(buffer, pattern) != buffer.len()
}

pub fn is_unique_bytes(data: &[u8]) -> bool {
    let mut items = ahash::AHashSet::with_capacity(data.len());
    for i in 0..data.len() {
        if !items.insert(data[i]) {
            return false;
        }
    }
    true
}
