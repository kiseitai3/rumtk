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
#[cfg(feature = "fast_allocator_options")]
use libmimalloc_sys as mimalloc_opts;


#[cfg(feature = "fast_allocator_options")]
pub struct MiMallocOpts {
    mem_os_alloc: usize,
    mem_huge_alloc: usize,
    mem_arena_alloc: usize,
    huge_pages: bool,
    large_pages: bool,
}

#[cfg(feature = "fast_allocator_options")]
impl Default for MiMallocOpts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "fast_allocator_options")]
impl MiMallocOpts {
    pub const fn new() -> Self {
        use crate::mem::constants::mimalloc_constants::*;
        Self {
            mem_os_alloc: DEFAULT_GLOBAL_MIMALLOC_ALLOCATION as usize,
            mem_huge_alloc: 1,
            mem_arena_alloc: (DEFAULT_GLOBAL_MIMALLOC_ALLOCATION / 2) as usize,
            huge_pages: true,
            large_pages: true,
        }
    }

    #[inline(always)]
    pub const fn builder() -> Self {
        Self::new()
    }

    #[inline(always)]
    pub const fn set_os_alloc(&mut self, mem_alloc: usize) -> &mut Self {
        self.mem_os_alloc = mem_alloc;
        self
    }

    #[inline(always)]
    pub const fn set_huge_alloc(&mut self, mem_alloc: usize) -> &mut Self {
        self.mem_huge_alloc = mem_alloc;
        self
    }

    #[inline(always)]
    pub const fn set_arena_alloc(&mut self, mem_alloc: usize) -> &mut Self {
        self.mem_arena_alloc = mem_alloc;
        self
    }

    #[inline(always)]
    pub const fn enable_huge_pages(&mut self) -> &mut Self {
        self.huge_pages = true;
        self
    }

    #[inline(always)]
    pub const fn enable_large_pages(&mut self) -> &mut Self {
        self.large_pages = true;
        self
    }

    #[inline(always)]
    pub fn apply(&self) -> &Self {
        use crate::mem::constants::mimalloc_constants::*;
        use std::ffi::c_long;
        unsafe {
            mimalloc_opts::mi_option_set_enabled(mimalloc_opts::mi_option_large_os_pages, self.large_pages);
            mimalloc_opts::mi_option_set_enabled(OPT_ALLOW_THP, self.huge_pages);
            mimalloc_opts::mi_option_set(mimalloc_opts::mi_option_reserve_huge_os_pages, self.mem_huge_alloc as c_long);
            mimalloc_opts::mi_option_set(OPT_ARENA_RESERVE, self.mem_arena_alloc as c_long);
            mimalloc_opts::mi_option_set(OPT_RESERVE_OS_MEMORY, self.mem_os_alloc as c_long);
        }
        self
    }

    #[inline(always)]
    pub fn enable_debug(&self) -> &Self {
        unsafe {
            mimalloc_opts::mi_option_set_enabled(mimalloc_opts::mi_option_show_stats, true);
            mimalloc_opts::mi_option_set_enabled(mimalloc_opts::mi_option_verbose, true);
        }
        self
    }
}
