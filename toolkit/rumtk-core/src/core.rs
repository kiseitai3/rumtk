/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */
use crate::strings::RUMString;
use compact_str::format_compact;
pub use smallvec::{smallvec, SmallVec};

///
/// Type used for propagating error messages.
///
pub type RUMResult<T> = Result<T, RUMString>;

pub type RUMVec<T> = Vec<T>;

pub fn is_unique<T: std::cmp::Eq + std::hash::Hash>(data: &Vec<T>) -> bool {
    let mut keys = ahash::AHashSet::with_capacity(data.len());
    for itm in data {
        if !keys.insert(itm) {
            return false;
        }
    }
    true
}

///
/// Take a requested index and the maximum size of the item container.
/// Check if the index is valid and return an error if it is.
/// The purpose of this function is to enable handling of out of bounds without triggering a panic.
/// Also, add negative indices like Python does when doing a reverse search!
///
/// * If the index is 0, return Error
/// * If the index is below 0, return the max - index iff max - index > 0
/// * If the index is bigger than the defined max, return Error.
/// * Otherwise, return the given index.
///
/// # Examples
///
/// ## Min
/// ```
/// use ::rumtk_core::core::clamp_index;
/// use ::rumtk_core::strings::format_compact;
/// let max: isize = 5;
/// let i: isize = 1;
/// let result = clamp_index(&i, &max).unwrap();
/// assert_eq!(&1, &result, "{}", format_compact!("Expected to receive 0 but got {}", &result))
/// ```
///
/// ## Max
/// ```
/// use ::rumtk_core::core::clamp_index;
/// use ::rumtk_core::strings::format_compact;
/// let max: isize = 5;
/// let i: isize = 5;
/// let result = clamp_index(&i, &max).unwrap();
/// assert_eq!(&5, &result, "{}", format_compact!("Expected to receive 0 but got {}", &result))
/// ```
///
/// ## Valid
/// ```
/// use ::rumtk_core::core::clamp_index;
/// use ::rumtk_core::strings::format_compact;
/// let max: isize = 5;
/// let i: isize = 5;
/// let result = clamp_index(&i, &max).unwrap();
/// assert_eq!(&5, &result, "{}", format_compact!("Expected to receive 0 but got {}", &result))
/// ```
///
/// ## Valid Negative Index (reverse lookup)
/// ```
/// use ::rumtk_core::core::clamp_index;
/// use ::rumtk_core::strings::format_compact;
/// let max: isize = 5;
/// let i: isize = -1;
/// let result = clamp_index(&i, &max).unwrap();
/// assert_eq!(&5, &result, "{}", format_compact!("Expected to receive 0 but got {}", &result))
/// ```
#[inline(always)]
pub fn clamp_index(given_indx: &isize, max_size: &isize) -> RUMResult<usize> {
    let neg_max_indx = *max_size * -1;
    if *given_indx == 0 {
        return Err(format_compact!(
            "Index {} is invalid! Use 1-indexed values if using positive indices.",
            given_indx
        ));
    }

    if *given_indx >= neg_max_indx && *given_indx < 0 {
        return Ok((max_size + given_indx + 1) as usize);
    }

    if *given_indx > 0 && given_indx <= max_size {
        return Ok(*given_indx as usize);
    }

    Err(format_compact!(
        "Index {} is outside {} < x < {} boundary!",
        given_indx,
        neg_max_indx,
        max_size
    ))
}
