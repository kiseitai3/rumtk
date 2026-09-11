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
use crate::base::{RUMResult, RUMVec};
use crate::buffers::buffer_to_str;
use crate::mem::{as_slice_mut, copy_from_slice, AsPtr, AsSlice, SizedType};
use crate::rumtk_layout;
use std::alloc::{alloc, dealloc, Layout};
use std::cmp::PartialEq;
use std::ops::DerefMut;
use std::ops::IndexMut;
use std::ops::{Deref, RangeTo, RangeToInclusive};
use std::ops::{Index, Range, RangeFull};
use std::sync::LazyLock;

const EMPTY_BUFFER_DATA: [u8;0] = [0;0];
static EMPTY_RUMBUFFER: LazyLock<RUMBuffer> = LazyLock::new(|| RUMBuffer::new());

///
/// The [RUMBuffer] type is meant to be a very lightweight owned buffer pointer. The impetus for building
/// this type is that benchmarking showed a potential vector getting allocated by the **bytes** crate.
/// I was using the bytes crate before because it is a very solid crate. However, the vector allocations worried me.
/// It was likely part of that crates interior mutability strategy with vtables but it was one of a few
/// areas consuming a lot of time and most importantly consuming kernel time by invoking malloc.
///
/// ## Example
/// ### Creation
/// ```
/// use rumtk_arena::buffers::*;
///
/// let buffer = RUMBuffer::from("Hello World!");
/// let expected = b"Hello World!";
///
/// assert_eq!(&buffer[..], expected, "Could not create RUMBuffer!");
/// ```
///
/// ### Split Buffer
/// ```
/// use rumtk_arena::buffers::*;
///
/// let mut buffer = RUMBuffer::from("Hello World!");
/// let section = buffer.split_to(5).unwrap();
/// let expected = b"Hello";
///
/// assert_eq!(&section[..], expected, "Could not create RUMBuffer!");
/// ```
///
#[derive(Debug)]
pub struct RUMBuffer {
    data: *const u8,
    size: u32,
    dealloc: bool,
}

impl RUMBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: EMPTY_BUFFER_DATA.as_ptr(),
            size: 0,
            dealloc: false,
        }
    }

    #[inline]
    pub fn from_slice(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::new();
        }

        let data_length = data.len();
        let ptr = unsafe { alloc(Layout::from_size_align_unchecked(data_length, size_of::<u8>())) };
        let dst = as_slice_mut(ptr, data_length);
        copy_from_slice(&data[..], dst);
        Self {
            data: ptr,
            size: data_length as u32,
            dealloc: true,
        }
    }

    #[inline]
    pub fn from_static(mut data: &'static [u8]) -> Self {
        let ptr = data.as_ptr();
        let size = data.len();
        Self {
            data: ptr,
            size: size as u32,
            dealloc: false,
        }
    }

    #[inline]
    pub fn from_parts(data: *const u8, length: usize, dealloc: bool) -> Self {
        Self {
            data,
            size: length as u32,
            dealloc,
        }
    }

    #[inline]
    pub fn split_to(&mut self, offset: usize) -> Option<Self> {
        if offset <= self.size as usize {
            let ptr = self.as_ptr();
            let new_ptr = unsafe { ptr.add(offset) };
            let copy = Self {
                data: ptr,
                size: offset as u32,
                dealloc: false,
            };
            self.data = new_ptr;
            self.size -= (offset as u32);
            Some(copy)
        } else {
            None
        }
    }

    #[inline]
    pub fn freeze(&self) -> Self {
        Self {
            data: self.as_ptr(),
            size: self.size,
            dealloc: false,
        }
    }

    #[inline]
    pub fn to_vec(&self) -> RUMVec<u8> {
        self.as_slice().to_vec()
    }

    #[inline]
    pub fn as_str(&self) -> RUMResult<&str> {
        buffer_to_str(self)
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.size = len as u32;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn is_buffer(&self) -> bool {
        self.dealloc
    }

    #[inline]
    pub fn is_view(&self) -> bool {
        !self.dealloc
    }
}

impl Iterator for RUMBuffer {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 { return None };

        let v = self[0];
        self.data = unsafe { self.as_ptr().add(1) };
        Some(v)
    }
}

impl SizedType for RUMBuffer {
    fn size(&self) -> usize {
        self.size as usize
    }
}

impl AsPtr for RUMBuffer {
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.data
    }
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data as *mut u8
    }
}

impl AsSlice for RUMBuffer {  }

impl AsRef<[u8]> for RUMBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for RUMBuffer {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for RUMBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl PartialEq for RUMBuffer {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Default for RUMBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RUMBuffer {
    #[inline]
    fn clone(&self) -> Self {
        self.freeze()
    }
}

impl Drop for RUMBuffer {
    #[inline]
    fn drop(&mut self) {
        if self.dealloc {
            unsafe {
                dealloc(self.as_mut_ptr(), rumtk_layout!(self.size as usize))
            }
        }
    }
}

unsafe impl Send for RUMBuffer {}
unsafe impl Sync for RUMBuffer {}

///////////////////// Indexing ////////////////////////////////////

impl Index<usize> for RUMBuffer {
    type Output = u8;
    #[inline]
    fn index(&self, i: usize) -> &Self::Output {
        &self.as_slice()[i]
    }
}

impl Index<Range<usize>> for RUMBuffer {
    type Output = [u8];
    #[inline]
    fn index(&self, i: Range<usize>) -> &Self::Output {
        &self.as_slice()[i.start..i.end]
    }
}

impl Index<RangeTo<usize>> for RUMBuffer {
    type Output = [u8];
    #[inline]
    fn index(&self, i: RangeTo<usize>) -> &Self::Output {
        &self.as_slice()[..i.end]
    }
}

impl Index<RangeToInclusive<usize>> for RUMBuffer {
    type Output = [u8];
    #[inline]
    fn index(&self, i: RangeToInclusive<usize>) -> &Self::Output {
        &self.as_slice()[..=i.end]
    }
}

impl Index<RangeFull> for RUMBuffer {
    type Output = [u8];
    #[inline]
    fn index(&self, i: RangeFull) -> &Self::Output {
        self.as_slice()
    }
}

///////////////////// Indexing (Mut) ////////////////////////////////////

impl IndexMut<usize> for RUMBuffer {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[i]
    }
}

impl IndexMut<Range<usize>> for RUMBuffer {
    #[inline]
    fn index_mut(&mut self, i: Range<usize>) -> &mut Self::Output {
        &mut self.as_slice_mut()[i.start..i.end]
    }
}

impl IndexMut<RangeTo<usize>> for RUMBuffer {
    #[inline]
    fn index_mut(&mut self, i: RangeTo<usize>) -> &mut Self::Output {
        &mut self.as_slice_mut()[..i.end]
    }
}

impl IndexMut<RangeToInclusive<usize>> for RUMBuffer {
    #[inline]
    fn index_mut(&mut self, i: RangeToInclusive<usize>) -> &mut Self::Output {
        &mut self.as_slice_mut()[..=i.end]
    }
}

impl IndexMut<RangeFull> for RUMBuffer {
    #[inline]
    fn index_mut(&mut self, i: RangeFull) -> &mut Self::Output {
        self.as_slice_mut()
    }
}

/////////////////////// Conversions ////////////////////////////////
impl From<String> for RUMBuffer {
    #[inline]
    fn from(mut data: String) -> Self {
        let (s, l, c) = data.into_raw_parts();
        Self::from_parts(s, l, true)
    }
}
impl From<Vec<u8>> for RUMBuffer {
    #[inline]
    fn from(mut data: Vec<u8>) -> Self {
        let (s, l, c) = data.into_raw_parts();
        Self::from_parts(s, l, true)
    }
}

impl From<&RUMVec<u8>> for RUMBuffer {
    #[inline]
    fn from(data: &RUMVec<u8>) -> Self {
        Self::from_slice(data.as_slice())
    }
}

impl From<&[u8]> for RUMBuffer {
    #[inline]
    fn from(data: &[u8]) -> Self {
        Self::from_slice(data)
    }
}

impl<const N: usize> From<&[u8; N]> for RUMBuffer {
    #[inline]
    fn from(data: &[u8; N]) -> Self {
        Self::from(data.as_slice())
    }
}

impl From<&str> for RUMBuffer {
    #[inline]
    fn from(data: &str) -> Self {
        Self::from(data.as_bytes())
    }
}