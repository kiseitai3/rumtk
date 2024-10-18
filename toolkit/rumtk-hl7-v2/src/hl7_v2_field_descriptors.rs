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

pub mod v2_field_descriptor {
    use crate::hl7_v2_base_types::v2_base_types::V2String;
    use crate::hl7_v2_base_types::v2_primitives::V2PrimitiveType;
    use ::phf::Map;
    use ::phf_macros::phf_map;

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

    pub type V2FieldDescriptor = Vec<&'static V2ComponentTypeDescriptor>;
    pub type V2FieldDescriptors = Map<&'static str, &'static V2FieldDescriptor>;

    #[macro_export]
    macro_rules! v2_component_descriptor {
        ( $name:expr, $data_type:expr, $max_input_len:expr, $seq:expr, $valid_table:expr, $required:expr, $truncate:expr ) => {{
            &V2ComponentTypeDescriptor::new(
                V2String::from($name),
                $data_type,
                $max_input_len,
                $seq,
                $valid_table,
                $required,
                $truncate,
            )
        }};
    }
    pub static V2_FIELD_DESCRIPTORS: V2FieldDescriptors = phf_map! {
        "AD" => &vec![
            v2_component_descriptor!("Street Address", V2PrimitiveType::V2ST, 120, 1, 0, false, true),
            v2_component_descriptor!("Other Designation", V2PrimitiveType::V2ST, 120, 2, 0, false, true),
            v2_component_descriptor!("City", V2PrimitiveType::V2ST, 50, 3, 0, false, true),
            v2_component_descriptor!("State or Province", V2PrimitiveType::V2ST, 50, 4, 0, false, true),
            v2_component_descriptor!("Zip or Postal Code", V2PrimitiveType::V2ST, 12, 5, 0, false, false),
            v2_component_descriptor!("Country", V2PrimitiveType::V2ID, 0, 6, 399, false, false),
            v2_component_descriptor!("Address Type", V2PrimitiveType::V2ID, 0, 7, 190, false, false),
            v2_component_descriptor!("Other Geographic Designation", V2PrimitiveType::V2ST, 50, 8, 0, false, true),
        ]
    };
}
