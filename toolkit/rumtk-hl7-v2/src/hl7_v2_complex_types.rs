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

pub mod hl7_v2_complex_types {
    use crate::hl7_v2_base_types::v2_primitives::*;

    type V2StrField = Vec<str>;

    ///
    /// Interface for ensuring we get a vector of strings instead of components.
    /// This ensures we keep this module independent of the parser module.
    ///
    pub trait V2FieldToString {
        fn to_component_list(&self) -> V2StrField;
    }

    pub struct V2ComponentType {
        name: V2String,
        data_type: V2PrimitiveTypes,
        max_input_len: u32,
        seq: u16,
        valid_table: u16,
        optional: bool,
        truncate: bool,
    }

    pub const fn validate_and_cast_component<T>(
        component: &str,
        component_type: &V2ComponentType,
    ) -> V2Result<T> {
        if component_type.optional && component.len() == 0 {
            //return
        }
        match component_type.data_type {}
    }
}
