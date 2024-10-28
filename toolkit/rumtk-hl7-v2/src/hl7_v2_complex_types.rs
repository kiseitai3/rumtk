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
    use crate::hl7_v2_field_descriptors::v2_field_descriptor::*;
    use rumtk_core::strings::format_compact;

    type V2StrField<'a> = Vec<&'a str>;

    ///
    /// Interface for ensuring we get a vector of strings instead of components.
    /// This ensures we keep this module independent of the parser module.
    ///
    pub trait V2FieldToString: Sized {
        fn to_component_list(&self) -> V2StrField;
    }

    #[derive(Debug)]
    pub enum V2Type {
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

    pub fn cast_component(
        component: Vec<&str>,
        component_type: &V2ComponentTypeDescriptor,
        characters: &V2ParserCharacters,
    ) -> V2Type {
        if component_type.optionality.is_required() && component.len() == 0 {
            return V2Type::Err(format_compact!(
                "Required data in seq {} is missing!",
                component_type.seq
            ));
        }
        match &component_type.data_type {
            V2ComponentType::Primitive(primitive) => {
                if component.len() > 1 {
                    V2Type::Err(
                        format_compact!(
                            "Received a tuple as component but components flagged with a primitive type \
                            expect only one string. Got [{:?}]", &component
                        )
                    )
                } else {
                    let c = component[0];
                    match primitive {
                        V2PrimitiveType::DateTime => V2Type::V2DateTime(c.to_v2datetime()),
                        V2PrimitiveType::Date => V2Type::V2Date(c.to_v2date()),
                        V2PrimitiveType::Time => V2Type::V2Time(c.to_v2time()),
                        V2PrimitiveType::FT => {
                            V2Type::V2FT(c.to_v2formattedtext(&characters.repetition_separator))
                        }
                        V2PrimitiveType::Text => {
                            V2Type::V2Text(c.to_v2text(&characters.repetition_separator))
                        }
                        V2PrimitiveType::String => V2Type::V2String(c.to_v2string()),
                        V2PrimitiveType::SNM => V2Type::V2SNM(c.to_v2telephonestring()),
                        V2PrimitiveType::ID => V2Type::V2ID(c.to_v2id()),
                        V2PrimitiveType::IS => V2Type::V2IS(c.to_v2is()),
                        V2PrimitiveType::NM => V2Type::V2NM(c.to_v2number()),
                        V2PrimitiveType::ST => V2Type::V2ST(c.to_v2stringdata()),
                        V2PrimitiveType::SI => V2Type::V2SI(c.to_v2sequenceid()),
                    }
                }
            }
            V2ComponentType::Complex(complex) => match complex {
                _ => V2Type::Err(format_compact!("Unknown requested type!")),
            },
        }
    }
}
