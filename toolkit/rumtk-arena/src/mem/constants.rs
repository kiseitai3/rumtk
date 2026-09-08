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
use std::alloc::Layout;

pub const NULL_U8_PTR: [u8;0] = [0u8;0];
pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;
pub const GB: usize = 1024 * 1024 * 1024;
pub const DEFAULT_GLOBAL_MB_ALLOCATION: usize = 50 * MB;
pub const DEFAULT_GLOBAL_MB_ALLOCATION_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(DEFAULT_GLOBAL_MB_ALLOCATION, size_of::<u8>()) };

#[cfg(feature = "fast_allocator_options")]
pub mod mimalloc_constants {
    use crate::KB;
    use std::ffi::{c_int, c_long};

    pub const DEFAULT_GLOBAL_MIMALLOC_ALLOCATION: c_long = (50 * KB) as c_long;
    pub const OPT_RESERVE_OS_MEMORY: c_int = 10;
    pub const OPT_ARENA_RESERVE: c_int = 23;
    pub const OPT_ALLOW_THP: c_int = 43;
}
