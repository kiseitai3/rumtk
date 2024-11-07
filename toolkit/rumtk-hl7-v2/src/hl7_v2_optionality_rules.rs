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

const CONDITION_NOOP: V2ComponentConditionFn = |c: &V2ComponentList| { false };

/***************CF Conditions**************/
///
/// As of v2.7 this component is required when CF.1 is populated and CF.14 is not populated. Both
/// CF.3 and CF.14 may be populated. Receivers should not identify a code based on its position
/// within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or position within
/// a repeating field. Instead, the receiver should always examine the coding system as specified in
/// CF.3 and/or CF.14, the Coding System component or the Coding System OID, for the tuple.
///
pub const CONDITION_CF1: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[0].len() > 0 && !c[13].len() > 0 };
///
/// As of v2.7 this component is required when CF.4 is populated and CF.17 is not populated. Both
/// CF.6 and CF.17 may be populated. Receivers should not identify a code based on its position
/// within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or position within
/// a repeating field. Instead, the receiver should always examine the coding system as specified in
/// CF.6 and/or CF.17, the Coding System component or the Coding System OID, for the tuple.
///
pub const CONDITION_CF2: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[3].len() > 0 && !c[16].len() > 0 };
///
/// Definition: This component carries the version for the coding system identified by components 1-
/// 3. If CF.3 is populated with a value other than HL7nnnn or is of table type user-defined, version
/// ID must be valued with an actual version ID. If CF.3 is populated with a value of HL7nnnn and
/// nnnn is of table type HL7, version ID may have an actual value or it may be absent. If version ID
/// is absent, it will be interpreted to have the same value as the HL7 version number in the message
/// header.
///
/// ## Note
///     ??? What. Not quiet implementable or worth the hassle of aborting validation.
///     Leaving this for a more global rule.
///
pub const CONDITION_CF3: V2ComponentConditionFn = CONDITION_NOOP;
///
/// ??? No real condition? Why even mark field as conditionally required?
///
pub const CONDITION_CF4: V2ComponentConditionFn = CONDITION_NOOP;
///
/// This component is required when CF.1 is populated and CF.3 is not populated. Both CF.3 and
/// CF.14 may be populated.
///
pub const CONDITION_CF5: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[0].len() > 0 && !c[2].len() > 0 };
///
/// Value set version ID is required if CF.15 is populated.
///
pub const CONDITION_CF6: V2ComponentConditionFn = |c: &V2ComponentList| { c[14].len() > 0 };
///
/// This component is required when CF.4 is populated and CF.6 is not populated. Both CF.6 and
/// CF.17 may be populated.
///
pub const CONDITION_CF7: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[3].len() > 0 && !c[5].len() > 0 };
///
/// Value set version ID is required if CF.18 is populated.
///
pub const CONDITION_CF8: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[17].len() > 0 };
///
/// This component is required when CF.10 is populated and CF.12 is not populated. Both CF.12 and
/// CF.20 may be populated.
///
pub const CONDITION_CF9: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[9].len() > 0 && !c[11].len() > 0 };
///
/// Value set version ID is required if CF.21 is populated.
///
pub const CONDITION_CF10: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[20].len() > 0 };

/***************CNE Conditions*************/
///
/// **Usage Note:** If the coding system is any system other than an "HL7 coding system," version ID
/// must be valued with an actual version ID. If the coding system is "HL7 coding system," version
/// ID may have an actual value or it may be absent. If version ID is absent, it will be interpreted to
/// have the same value as the HL7 version number in the message header. Text description of code is
/// optional but its use should be encouraged since it makes messages easier to review for accuracy,
/// especially during interface testing and debugging.
///
/// ## Note
///     ???? Not sure how to implement this one without a lot of global state which I am trying to
///     avoid. For now, defaulting to a noop.
///
pub const CONDITION_CNE1: V2ComponentConditionFn = CONDITION_NOOP;
///
/// See [CONDITION_CNE1]
///
pub const CONDITION_CNE2: V2ComponentConditionFn = CONDITION_NOOP;
///
/// This component is required when CNE.1 is populated and CNE.3 is not populated. Both CNE.3
/// and CNE.14 may be populated.
///
pub const CONDITION_CNE3: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[0].len() > 0 && !(c[2].len() > 0) };
///
/// Value set version ID is required if CNE.15 is populated.
///
pub const CONDITION_CNE4: V2ComponentConditionFn = |c: &V2ComponentList| { c[14].len() > 0 };
///
/// This component is required when CNE.4 is populated and CNE.6 is not populated. Both CNE.6
/// and CNE.17 may be populated.
///
pub const CONDITION_CNE5: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[3].len() > 0 && !(c[5].len() > 0) };
///
/// Value set version ID is required if CNE.18 is populated.
///
pub const CONDITION_CNE6: V2ComponentConditionFn = |c: &V2ComponentList| c[17].len() > 0;
///
/// This component is required when CNE.4 is populated and neither CNE.6 nor CNE.18 is populated.
/// In short either the CNE.6 or the CNE.14 or CNE.17 must be populated when CNE.4 is populated.
///
pub const CONDITION_CNE7: V2ComponentConditionFn =
    |c: &V2ComponentList| { c[3].len() > 0 && !(c[5].len() > 0 || c[17].len() > 0) };
///
/// Value set version ID is required if CNE.21 is populated.
///
pub const CONDITION_CNE8: V2ComponentConditionFn = |c: &V2ComponentList| { c[20].len() > 0 };

/***************CNN Conditions*************/

///
/// If component 1 is valued, either CNN.8 or CNN.9, or both CNN.10 and CNN.11, must be valued.
///
pub const CONDITION_CNN1: V2ComponentConditionFn =
    |c: &V2ComponentList| { (c[0].len() > 0 && c[8].len() > 0) || !CONDITION_CNN2(&c) };

///
/// If component 1 is valued, either CNN.8 or CNN.9, or both CNN.10 and CNN.11, must be valued.
///
pub const CONDITION_CNN2: V2ComponentConditionFn =
    |c: &V2ComponentList| { (c[0].len() > 0 && c[7].len() > 0) || !CONDITION_CNN1(&c) };

///
/// If CNN.11 is valued, this component must be valued
///
pub const CONDITION_CNN3: V2ComponentConditionFn = |c: &V2ComponentList| { c[10].len() > 0 };

///
/// If CNN.10 is valued, this component must be valued.
///
pub const CONDITION_CNN4: V2ComponentConditionFn = |c: &V2ComponentList| { c[9].len() > 0 };
