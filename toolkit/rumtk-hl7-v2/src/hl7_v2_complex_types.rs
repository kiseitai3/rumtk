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
    use rumtk_core::strings::format_compact;

    type V2StrField<'a> = Vec<&'a str>;

    ///
    /// Interface for ensuring we get a vector of strings instead of components.
    /// This ensures we keep this module independent of the parser module.
    ///
    pub trait V2FieldToString: Sized {
        fn to_component_list(&self) -> V2StrField;
    }
    #[repr(C)]
    pub enum V2ComponentType {
        V2String(V2Result<V2String>),
        V2DateTime(V2Result<V2DateTime>),
        V2Date(V2Result<V2Date>),
        V2Time(V2Result<V2Time>),
        V2FT(V2Result<V2FT>),
        V2SNM(V2Result<V2SNM>),
        V2NM(V2Result<V2NM>),
        V2ID(V2Result<V2ID>),
        V2IS(V2Result<V2IS>),
        V2ST(V2Result<V2ST>),
        V2Text(V2Result<V2TX>),
        V2SI(V2Result<V2SI>),
        Err(V2String),
    }

    #[derive(Debug, Default)]
    pub struct V2ComponentTypeDescriptor {
        name: V2String,
        data_type: V2PrimitiveType,
        max_input_len: u32,
        seq: u16,
        valid_table: u16,
        required: bool,
        truncate: bool,
    }

    impl V2ComponentTypeDescriptor {
        pub fn new(
            name: V2String,
            data_type: V2PrimitiveType,
            max_input_len: u32,
            seq: u16,
            valid_table: u16,
            required: bool,
            truncate: bool,
        ) -> V2ComponentTypeDescriptor {
            V2ComponentTypeDescriptor {
                name,
                data_type,
                max_input_len,
                seq,
                valid_table,
                required,
                truncate,
            }
        }
    }

    pub fn cast_component(
        component: &str,
        component_type: &V2ComponentTypeDescriptor,
        characters: &V2ParserCharacters,
    ) -> V2ComponentType {
        if component_type.required && component.len() == 0 {
            return V2ComponentType::Err(format_compact!(
                "Required data in seq {} is missing!",
                component_type.seq
            ));
        }
        match component_type.data_type {
            V2PrimitiveType::V2DateTime => V2ComponentType::V2DateTime(component.to_v2datetime()),
            V2PrimitiveType::V2Date => V2ComponentType::V2Date(component.to_v2date()),
            V2PrimitiveType::V2Time => V2ComponentType::V2Time(component.to_v2time()),
            V2PrimitiveType::V2FT => V2ComponentType::V2FT(
                component.to_v2formattedtext(&characters.repetition_separator),
            ),
            V2PrimitiveType::V2Text => {
                V2ComponentType::V2Text(component.to_v2text(&characters.repetition_separator))
            }
            V2PrimitiveType::V2String => V2ComponentType::V2String(component.to_v2string()),
            V2PrimitiveType::V2SNM => V2ComponentType::V2SNM(component.to_v2telephonestring()),
            V2PrimitiveType::V2ID => V2ComponentType::V2ID(component.to_v2id()),
            V2PrimitiveType::V2IS => V2ComponentType::V2IS(component.to_v2is()),
            V2PrimitiveType::V2NM => V2ComponentType::V2NM(component.to_v2number()),
            V2PrimitiveType::V2ST => V2ComponentType::V2ST(component.to_v2stringdata()),
            V2PrimitiveType::V2SI => V2ComponentType::V2SI(component.to_v2sequenceid()),
        }
    }
}
