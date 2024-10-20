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
    use crate::hl7_v2_base_types::v2_primitives::V2PrimitiveType;
    pub use once_cell::unsync::Lazy;
    use ::phf::Map;
    use ::phf_macros::phf_map;

    #[derive(Debug, Default)]
    pub struct V2ComponentTypeDescriptor {
        pub name: &'static str,
        pub description: &'static str,
        pub data_type: V2PrimitiveType,
        pub max_input_len: u32,
        pub seq: u16,
        pub valid_table: u16,
        pub required: bool,
        pub truncate: bool,
    }

    impl V2ComponentTypeDescriptor {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            data_type: V2PrimitiveType,
            max_input_len: u32,
            seq: u16,
            valid_table: u16,
            required: bool,
            truncate: bool,
        ) -> V2ComponentTypeDescriptor {
            V2ComponentTypeDescriptor {
                name,
                description,
                data_type,
                max_input_len,
                seq,
                valid_table,
                required,
                truncate,
            }
        }
    }

    pub type V2FieldDescriptor = [&'static V2ComponentTypeDescriptor];
    pub type V2FieldDescriptors = Map<&'static str, &'static V2FieldDescriptor>;

    ///
    /// Generates instance of V2ComponentDescriptor which defines how we should cast a field.
    ///
    /// ## Arguments
    /// * `name` - String representing the component name.
    /// * `description` - String describing the component as given in the data type table.
    /// * `data_type` - Appropriate [`V2PrimitiveType`] enumerator item describing the type we should target when casting the component
    /// * `max_input_len` - Length to truncate value of component if `truncate` is True
    /// * `seq` - Number of component/sequence in field.
    /// * `valid_table` - Validation table used for additional validation of input. It's a number now, but may be changed to an enumerator in the future.
    /// * `required` - Boolean flag for marking component as required. If a required component is missing, emit error.
    /// * `truncate` - Boolean flag for marking the component as one that needs to be truncated to `max_input_len`.
    ///
    #[macro_export]
    macro_rules! v2_component_descriptor {
        ( $name:expr, $description:expr, $data_type:expr, $max_input_len:expr, $seq:expr, $valid_table:expr, $required:expr, $truncate:expr ) => {{
            &V2ComponentTypeDescriptor::new(
                $name,
                $description,
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
        "AD" => &[
            v2_component_descriptor!("street_address", "Street Address", V2PrimitiveType::V2ST, 120, 1, 0, false, true),
            v2_component_descriptor!("second_address", "Other Designation", V2PrimitiveType::V2ST, 120, 2, 0, false, true),
            v2_component_descriptor!("city", "City", V2PrimitiveType::V2ST, 50, 3, 0, false, true),
            v2_component_descriptor!("state", "State or Province", V2PrimitiveType::V2ST, 50, 4, 0, false, true),
            v2_component_descriptor!("zip", "Zip or Postal Code", V2PrimitiveType::V2ST, 12, 5, 0, false, false),
            v2_component_descriptor!("country", "Country", V2PrimitiveType::V2ID, 0, 6, 399, false, false),
            v2_component_descriptor!("address_type", "Address Type", V2PrimitiveType::V2ID, 0, 7, 190, false, false),
            v2_component_descriptor!("county", "Other Geographic Designation", V2PrimitiveType::V2ST, 50, 8, 0, false, true)
        ],
        "AUI" => &[
            v2_component_descriptor!("auth_number", "Authorization Number", V2PrimitiveType::V2ST, 30, 1, 0, false, false),
            v2_component_descriptor!("date", "Date", V2PrimitiveType::V2Date, 0, 2, 0, false, false),
            v2_component_descriptor!("source", "Source", V2PrimitiveType::V2ST, 199, 3, 0, false, true)
        ],
        "CCD" => &[
            v2_component_descriptor!("event", "Invocation Event", V2PrimitiveType::V2ID, 0, 1, 0, true, false),
            v2_component_descriptor!("date", "Date/time", V2PrimitiveType::V2DateTime, 0, 2, 100, false, false)
        ],
        "CCP" => &[
            v2_component_descriptor!("cc_factor", "Channel Calibration Sensitivity Correction Factor", V2PrimitiveType::V2NM, 6, 1, 0, false, true),
            v2_component_descriptor!("cc_baseline", "Channel Calibration Baseline", V2PrimitiveType::V2NM, 6, 2, 0, false, true),
            v2_component_descriptor!("cc_time_skew", "Channel Calibration Time Skew", V2PrimitiveType::V2NM, 6, 3, 0, false, true)
        ]
    };

    ///
    /// Enumerator listing every complex type we have defined so far. Complex type definitions here
    /// will be used to guide type casting of the string components of a field into the proper primitive
    /// component types and overall field structure.
    ///
    pub enum V2ComplexType {
        ///
        /// # 2A.3.1AD - address
        ///
        /// **Definition:** This data type specifies the address of a person, place or organization.
        ///
        /// **Note:** Used only in the LA1 data type. Retained for backward compatibility as of v2.6. Replaced elsewhere
        /// by the XAD data type as of v2.3.
        ///
        /// ## Example:
        ///     |10 ASH LN^#3^LIMA^OH^48132|
        ///
        /// ## 2A.3.1.1 Street Address (ST)
        ///     Definition: This component specifies the street or mailing address of a person or institution. When
        ///     referencing an institution, this first component is used to specify the institution name. When used
        ///     in connection with a person, this component specifies the first line of the address.
        ///
        /// ## 2A.3.1.2 Other Designation (ST)
        ///     Definition: This component specifies the second line of address. In general, it qualifies address.
        ///     Examples: Suite 555 or Fourth Floor. When referencing an institution, this component specifies
        ///     the street address.
        ///
        /// ## 2A.3.1.3 City (ST)
        ///     Definition: This component specifies the city, district or place where the addressee is located
        ///     depending upon the national convention for formatting addresses for postal usage.
        ///
        /// ## 2A.3.1.4 State or Province (ST)
        ///     Definition: This component specifies the state or province where the addressee is located. State or
        ///     province should be represented by the official postal service codes for that country.
        ///
        /// ## 2A.3.1.5 Zip or Postal Code (ST)
        ///     Definition: This component specifies the zip or postal code where the addressee is located. Zip or
        ///     postal codes should be represented by the official codes for that country. In the US, the zip code
        ///     takes the form 99999[-9999], while the Canadian postal code takes the form A9A9A9 and the
        ///     Australian Postcode takes the form 9999.
        ///
        /// ## 2A.3.1.6 Country (ID)
        ///     Definition: This component specifies the country where the addressee is located. HL7 specifies
        ///     that the 3-character (alphabetic) form of ISO 3166 be used for the country code. Refer to HL7
        ///     Table 0399 - Country Code in Chapter 2C, Code Tables, for valid values.
        ///
        /// ## 2A.3.1.7 Address Type (ID)
        ///     Definition: This component specifies the kind or type of address. Refer to HL7 Table 0190 -
        ///     Address Type in Chapter 2C, Code Tables, for valid values.
        ///
        /// ## 2A.3.1.8 Other Geographic Designation (ST)
        ///     Definition: This component specifies any other geographic designation that may be necessary. It
        ///     includes county, bioregion, SMSA, etc.
        AD,
        ///
        /// # 2A.3.2 AUI - authorization information
        ///
        /// **Definition:** This data type specifies the identifier or code for an insurance authorization instance
        /// and its associated detail.
        ///
        /// **Note:** Replaces the CM data type used in sections 6.5.6.14 IN1-14, as of v2.5.
        ///
        /// ## 2A.3.2.1 Authorization Number (ST)
        ///     Definition: Identifier assigned to the authorization.
        ///
        /// ## 2A.3.2.2 Date (DT)
        ///     Definition: Date of authorization.
        ///
        /// ## 2A.3.2.3 Source (ST)
        ///     Definition: Source of authorization.
        ///
        AUI,
        ///
        /// Definition: Specifies whether a charge action is based on an invocation event or is time-based.
        ///
        /// Note: Replaces the CM data type used in section 4.5.2.1 BLG-1, as of v2.5.
        ///
        /// ## 2A.3.3.1 Invocation Event (ID)
        ///     Definition: Specifies the code for the event precipitating/triggering the charge activity. Refer to
        ///     HL7 Table 0100 - Invocation event for valid values.
        ///
        /// ## 2A.3.3.2 Date/time (DTM)
        ///     Definition: The second component is used to express the exact time to charge for the ordered
        ///     service; it is used only when the CCD.1 value is T. When used, it is expressed as a DTM data type.
        ///
        CCD,
        ///
        /// # 2A.3.4 CCP - channel calibration parameters
        ///
        /// **Attention: Retained for backward compatibility only in version 2.7.** This is used only in the
        /// CD Channel Definition data type, which has been retained for backward compatibility only in
        /// v2.7.
        ///
        /// Definition: This data type identifies the corrections to channel sensitivity, the baseline, and the
        /// channel time skew when transmitting waveform results.
        ///
        /// Note: Replaces the CM data type used in 7.14.1.5 OBX-5.3 where OBX-5 Observation value (*) is data
        /// type CD as of v 2.5.
        ///
        /// ## 2A.3.4.1 Channel Calibration Sensitivity Correction Factor (NM)
        ///     Definition: This component defines a correction factor for channel sensitivity, which may be
        ///     derived from the last calibration procedure performed. The actual channel sensitivity is the
        ///     nominal channel sensitivity given in the previous component multiplied by the unitless correction
        ///     factor.
        ///
        /// ## 2A.3.4.2 Channel Calibration Baseline (NM)
        ///     Definition: This component defines the actual channel baseline (the data value which corresponds
        ///     to a nominal input signal of zero). The actual baseline may differ from the ideal because of a dc
        ///     offset in the amplifier connected to the ADC. The actual baseline values for all channels (which
        ///     need not be integers) may be determined at the time of calibration as the average digitized values
        ///     obtained when a zero input signal is connected to each channel.
        ///
        /// ## 2A.3.4.3 Channel Calibration Time Skew (NM)
        ///     Definition: This component defines the time difference between the nominal sampling
        ///     (digitization) time (which would be the same for all channels) and the actual sampling time of the
        ///     channel, in seconds (or fractions thereof). This value will differ from zero when all channels in the
        ///     montage are not sampled simultaneously, as occurs in systems, which sample successive channels
        ///     at regular time intervals. This value may be determined from a calibration procedure in which an
        ///     identical time-varying signal is applied to all channels and interchannel time differences are
        ///     estimated, or more commonly it may be taken from the manufacturer’s specifications for the
        ///     digitizing system used. For example, for a system which samples successive channels at regular
        ///     time intervals t, the time skew of channel number n would be (n-1)t. The actual time of sampling
        ///     (digitization) of sample number m of channel number n in such a system would be R + (m-1)/f +
        ///     (n-1)t, where R is the reference time at the start of the epoch and f is the channel sampling
        ///     frequency (t < 1/f).
        ///
        CCP,
    }

    ///
    /// Return string key corresponding to enumerator key.
    ///
    pub fn complex_type_to_str(complex_type: &V2ComplexType) -> &str {
        match complex_type {
            V2ComplexType::AD => "AD",
            V2ComplexType::AUI => "AUI",
            V2ComplexType::CCD => "CCD",
            V2ComplexType::CCP => "CCP",
        }
    }
}
