/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
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

use crate::hl7_v2_base_types::v2_primitives::V2ComponentList;

pub type V2ComponentConditionFn = fn(field: &V2ComponentList) -> bool;

///
///
///
#[derive(Debug)]
pub enum Optionality {
    /// Required
    R,
    /// Required but may be empty
    RE,
    /// Undeclared Conditional if None, Declared Conditional if filled vector (C(a|b)).
    C(V2ComponentConditionFn),
    /// Not supported
    X,
    /// Optional
    O,
    /// Backwards Compatible
    B,
}

impl Optionality {
    ///
    /// Checks if this instance of Optionality is flagged as Required. Meaning containing
    /// component is flagged as required.
    ///
    pub fn is_required(&self) -> bool {
        match &self {
            Optionality::R => true,
            _ => false,
        }
    }

    ///
    /// Executes contained function in conditional if any and returns result.
    /// Otherwise, returns false. Meaning, this method will always succeed for
    /// non-conditional components. Yields whether component is required.
    ///
    pub fn is_conditionally_required(&self, field: &V2ComponentList) -> bool {
        match &self {
            Optionality::C(f) => f(&field),
            _ => false,
        }
    }
}

/******************************* Conditions ********************************/

///
/// If component 1 is valued, either CNN.8 or CNN.9, or both CNN.10 and CNN.11, must be valued.
///
pub const CONDITION_CNN1: V2ComponentConditionFn = |c: &V2ComponentList| {
    let sub_count = c.len();
    return sub_count > 8 && c[0].len() > 0 && c[8].len() > 0;
};

///
/// If component 1 is valued, either CNN.8 or CNN.9, or both CNN.10 and CNN.11, must be valued.
///
pub const CONDITION_CNN2: V2ComponentConditionFn = |c: &V2ComponentList| {
    let sub_count = c.len();
    return sub_count > 9 && c[0].len() > 0 && c[9].len() > 0;
};

///
/// If CNN.11 is valued, this component must be valued
///
pub const CONDITION_CNN3: V2ComponentConditionFn = |c: &V2ComponentList| {
    return c.len() >= 11 && c[11].len() > 0;
};

///
/// If CNN.10 is valued, this component must be valued.
///
pub const CONDITION_CNN4: V2ComponentConditionFn = |c: &V2ComponentList| {
    return c.len() >= 10 && c[10].len() > 0;
};
