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

    ///
    /// Enumerator listing every complex type we have defined so far. Complex type definitions here
    /// will be used to guide type casting of the string components of a field into the proper primitive
    /// component types and overall field structure.
    ///
    #[derive(Debug)]
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
        ///
        /// # 2A.3.5 CD - channel definition
        ///
        /// **Attention: Retained for backward compatibility onlyas of v 2.7.** This is used only in the
        /// waveform message, CHM category, which has been retained for backward compatibility only in v
        /// 2.7.
        ///
        /// Definition: This data type is used for labeling of digital waveform data. It defines a recording
        /// channel, which is associated with one of the values in each time sample of waveform data. Each
        /// channel has a number (which generally defines its position in a multichannel display) and an
        /// optional name or label (also used in displays). One or two named waveform sources may also be
        /// associated with a channel (providing for the use of differential amplifiers with two inputs). The
        /// other components of the channel definition data type are optional. The individual components are
        /// defined as follows:
        ///
        /// ## 2A.3.5.1 Channel Identifier (WVI)
        ///     Definition: This component specifies the number and name of the recording channel where
        ///     waveform data is transmitted.
        ///
        /// ## 2A.3.5.2 Waveform Source (WVS)
        ///     Definition: This component identifies the source of the waveform connected to the channel. Two
        ///     names may be specified if it is necessary to individually identify the two inputs for a waveform.
        ///     Only one name need be specified if the channel is connected to a single input. For example, in
        ///     EKG recordings typically only one name is used (such as I or II); in electroencephalography, two
        ///     names are typically used, one for each input of the differential amplifier (such as F3 and C3).
        ///
        /// ## 2A.3.5.3 Channel Sensitivity and Units (CSU)
        ///     Definition: This component defines the channel sensitivity (gain) and the units in which it is
        ///     measured.
        ///
        /// ## 2A.3.5.4 Channel Calibration Parameters (CCP)
        ///     Definition: This component identifies the corrections to channel sensitivity, the baseline, and the
        ///     channel time skew.
        ///
        /// ## 2A.3.5.5 Channel Sampling Frequency (NM)
        ///     Definition: This component defines the sampling frequency in hertz of the channel, that is, the
        ///     reciprocal of the time in seconds between successive samples
        ///
        /// ## 2A.3.5.6 Minimum and Maximum Data Values (NR)
        /// **Note:** this is the frequency of transmitted data, which may or may not be the actual frequency at which the
        /// data was acquired by an analog-to-digital converter or other digital data source (i.e. the data transmitted
        /// may be subsampled, or interpolated, from the originally acquired data.)
        ///
        ///     Definition: This component defines the minimum and maximum data values which can occur in
        ///     this channel in the digital waveform data, that is, the range of the ADC. , and also specifies
        ///     whether or not non-integral data values may occur in this channel in the waveform data. If the
        ///     minimum and maximum values are both integers (or not present), only integral data values may be
        ///     used in this channel. If either the minimum or the maximum value contains a decimal point, then
        ///     non-integral as well as integral data values may be used in this channel. For an n-bit signed ADC,
        ///     the nominal baseline B = 0, and the minimum (L) and maximum (H) values may be calculated as
        ///     follows:
        ///         L = -2n-1
        ///         H = 2n-1 - 1
        ///
        ///     For an unsigned n-bit ADC, the minimum value L = 0, and the nominal baseline value (B) and
        ///     maximum value (H) may be calculated from the formulas,
        ///         B = 2n-1
        ///         H = 2n - 1
        ///
        ///     The actual signal amplitude A (for differentially amplified potential measurements, the potential at
        ///     electrode number one minus that at electrode number two) may be calculated from the value D
        ///     (range L to H) in the waveform data using the actual baseline value B and the nominal sensitivity
        ///     S and actual sensitivity correction factor C by the formula,
        ///
        ///         A = SC(D-B)
        ///
        CD,
        CSU,
        NR,
        WVI,
        WVS,
    }

    #[derive(Debug)]
    pub enum V2FieldType {
        Primitive(V2PrimitiveType),
        Complex(V2ComplexType),
    }

    #[derive(Debug)]
    pub struct V2FieldTypeDescriptor {
        pub name: &'static str,
        pub description: &'static str,
        pub data_type: V2FieldType,
        pub max_input_len: u32,
        pub seq: u16,
        pub valid_table: u16,
        pub required: bool,
        pub truncate: bool,
    }

    impl V2FieldTypeDescriptor {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            data_type: V2FieldType,
            max_input_len: u32,
            seq: u16,
            valid_table: u16,
            required: bool,
            truncate: bool,
        ) -> V2FieldTypeDescriptor {
            V2FieldTypeDescriptor {
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

    pub type V2FieldDescriptor = [&'static V2FieldTypeDescriptor];
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
    macro_rules! v2_field_descriptor {
        ( $name:expr, $description:expr, $data_type:expr, $max_input_len:expr, $seq:expr, $valid_table:expr, $required:expr, $truncate:expr ) => {{
            &V2FieldTypeDescriptor::new(
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
            v2_field_descriptor!("street_address", "Street Address", V2FieldType::Primitive(V2PrimitiveType::ST), 120, 1, 0, false, true),
            v2_field_descriptor!("second_address", "Other Designation", V2FieldType::Primitive(V2PrimitiveType::ST), 120, 2, 0, false, true),
            v2_field_descriptor!("city", "City", V2FieldType::Primitive(V2PrimitiveType::ST), 50, 3, 0, false, true),
            v2_field_descriptor!("state", "State or Province", V2FieldType::Primitive(V2PrimitiveType::ST), 50, 4, 0, false, true),
            v2_field_descriptor!("zip", "Zip or Postal Code", V2FieldType::Primitive(V2PrimitiveType::ST), 12, 5, 0, false, false),
            v2_field_descriptor!("country", "Country", V2FieldType::Primitive(V2PrimitiveType::ID), 0, 6, 399, false, false),
            v2_field_descriptor!("address_type", "Address Type", V2FieldType::Primitive(V2PrimitiveType::ID), 0, 7, 190, false, false),
            v2_field_descriptor!("county", "Other Geographic Designation", V2FieldType::Primitive(V2PrimitiveType::ST), 50, 8, 0, false, true)
        ],
        "AUI" => &[
            v2_field_descriptor!("auth_number", "Authorization Number", V2FieldType::Primitive(V2PrimitiveType::ST), 30, 1, 0, false, false),
            v2_field_descriptor!("date", "Date", V2FieldType::Primitive(V2PrimitiveType::Date), 0, 2, 0, false, false),
            v2_field_descriptor!("source", "Source", V2FieldType::Primitive(V2PrimitiveType::ST), 199, 3, 0, false, true)
        ],
        "CCD" => &[
            v2_field_descriptor!("event", "Invocation Event", V2FieldType::Primitive(V2PrimitiveType::ID), 0, 1, 0, true, false),
            v2_field_descriptor!("date", "Date/time", V2FieldType::Primitive(V2PrimitiveType::DateTime), 0, 2, 100, false, false)
        ],
        "CCP" => &[
            v2_field_descriptor!("cc_factor", "Channel Calibration Sensitivity Correction Factor", V2FieldType::Primitive(V2PrimitiveType::NM), 6, 1, 0, false, true),
            v2_field_descriptor!("cc_baseline", "Channel Calibration Baseline", V2FieldType::Primitive(V2PrimitiveType::NM), 6, 2, 0, false, true),
            v2_field_descriptor!("cc_time_skew", "Channel Calibration Time Skew", V2FieldType::Primitive(V2PrimitiveType::NM), 6, 3, 0, false, true)
        ],
        "CD" => &[
            v2_field_descriptor!("channel_id", "Channel Identifier", V2FieldType::Complex(V2ComplexType::WVI), 0, 1, 0, false, false),
            v2_field_descriptor!("waveform_source", "Waveform Source", V2FieldType::Complex(V2ComplexType::WVS), 0, 2, 0, false, false),
            v2_field_descriptor!("channel_sensitivity_units", "Channel Sensitivity and Units", V2FieldType::Complex(V2ComplexType::CSU), 0, 3, 0, false, false),
            v2_field_descriptor!("channel_calibration_parameters", "Channel Calibration Parameters", V2FieldType::Complex(V2ComplexType::CCP), 0, 4, 0, false, false),
            v2_field_descriptor!("channel_sampling_frequency", "Channel Sampling Frequency", V2FieldType::Primitive(V2PrimitiveType::NM), 6, 5, 0, false, true),
            v2_field_descriptor!("min_max_values", "Minimum and Maximum Data Values", V2FieldType::Complex(V2ComplexType::NR), 0, 6, 0, false, false)
        ]
    };

    ///
    /// Return string key corresponding to enumerator key.
    ///
    pub fn complex_type_to_str(complex_type: &V2ComplexType) -> &str {
        match complex_type {
            V2ComplexType::AD => "AD",
            V2ComplexType::AUI => "AUI",
            V2ComplexType::CCD => "CCD",
            V2ComplexType::CCP => "CCP",
            V2ComplexType::CD => "CD",
            _ => "Error",
        }
    }
}
