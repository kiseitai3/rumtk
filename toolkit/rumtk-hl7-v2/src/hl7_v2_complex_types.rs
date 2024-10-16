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
    use rumtk_core::strings::RUMStringConversions;

    type V2StrField<'a> = Vec<&'a str>;

    ///
    /// Interface for ensuring we get a vector of strings instead of components.
    /// This ensures we keep this module independent of the parser module.
    ///
    pub trait V2FieldToString: Sized {
        fn to_component_list(&self) -> V2StrField;
    }

    #[derive(Debug, Default)]
    pub struct V2ComponentType {
        name: V2String,
        data_type: V2PrimitiveType,
        max_input_len: u32,
        seq: u16,
        valid_table: u16,
        optional: bool,
        truncate: bool,
    }

    impl V2ComponentType {
        pub fn new(
            name: V2String,
            data_type: V2PrimitiveType,
            max_input_len: u32,
            seq: u16,
            valid_table: u16,
            optional: bool,
            truncate: bool,
        ) -> V2ComponentType {
            V2ComponentType {
                name,
                data_type,
                max_input_len,
                seq,
                valid_table,
                optional,
                truncate,
            }
        }
    }

    pub fn validate_and_cast_component<T: Default>(
        component: &str,
        component_type: &V2ComponentType,
        characters: &V2ParserCharacters,
    ) -> V2Result<T> {
        if component_type.optional && component.len() == 0 {
            return Ok(T::default());
        }
        match component_type.data_type {
            V2PrimitiveType::V2DateTime => Ok(component.to_v2datetime()),
            V2PrimitiveType::V2FT => {
                Ok(component.to_v2formattedtext(&characters.repetition_separator))
            }
            _ => Err("Error".to_rumstring()),
        }
    }
}
