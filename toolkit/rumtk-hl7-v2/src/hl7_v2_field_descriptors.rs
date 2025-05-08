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
    pub use crate::hl7_v2_optionality_rules::*;
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
        /// ```text
        ///     |10 ASH LN^#3^LIMA^OH^48132|
        /// ```
        /// ## 2A.3.1.1 Street Address (ST)
        /// ```text
        ///     Definition: This component specifies the street or mailing address of a person or institution. When
        ///     referencing an institution, this first component is used to specify the institution name. When used
        ///     in connection with a person, this component specifies the first line of the address.
        /// ```
        /// ## 2A.3.1.2 Other Designation (ST)
        /// ```text
        ///     Definition: This component specifies the second line of address. In general, it qualifies address.
        ///     Examples: Suite 555 or Fourth Floor. When referencing an institution, this component specifies
        ///     the street address.
        /// ```
        /// ## 2A.3.1.3 City (ST)
        /// ```text
        ///     Definition: This component specifies the city, district or place where the addressee is located
        ///     depending upon the national convention for formatting addresses for postal usage.
        /// ```
        /// ## 2A.3.1.4 State or Province (ST)
        /// ```text
        ///     Definition: This component specifies the state or province where the addressee is located. State or
        ///     province should be represented by the official postal service codes for that country.
        /// ```
        /// ## 2A.3.1.5 Zip or Postal Code (ST)
        /// ```text
        ///     Definition: This component specifies the zip or postal code where the addressee is located. Zip or
        ///     postal codes should be represented by the official codes for that country. In the US, the zip code
        ///     takes the form 99999[-9999], while the Canadian postal code takes the form A9A9A9 and the
        ///     Australian Postcode takes the form 9999.
        /// ```
        /// ## 2A.3.1.6 Country (ID)
        /// ```text
        ///     Definition: This component specifies the country where the addressee is located. HL7 specifies
        ///     that the 3-character (alphabetic) form of ISO 3166 be used for the country code. Refer to HL7
        ///     Table 0399 - Country Code in Chapter 2C, Code Tables, for valid values.
        /// ```
        /// ## 2A.3.1.7 Address Type (ID)
        /// ```text
        ///     Definition: This component specifies the kind or type of address. Refer to HL7 Table 0190 -
        ///     Address Type in Chapter 2C, Code Tables, for valid values.
        /// ```
        /// ## 2A.3.1.8 Other Geographic Designation (ST)
        /// ```text
        ///     Definition: This component specifies any other geographic designation that may be necessary. It
        ///     includes county, bioregion, SMSA, etc.
        /// ```
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
        ///
        /// # 2A.3.6 WITHDRAWN (CE – coded entry)
        ///
        /// **Attention: The CE data type has been replaced by the CWE and CNE data types and the detail
        /// was withdrawn and removed from the standard as of v 2.6.**
        ///
        CE,
        ///
        /// # 2A.3.7 CF - coded element with formatted values
        ///
        /// As of v2.7 a third tuple, formerly known as triplet, has been added to the CF data type.
        /// Additionally, 3 new components were added to each tuple such that each tuple now has a total of 7
        /// components. The Original Text component applies to the CF as a whole.
        ///
        /// **Note:** The Vocabulary TC is the steward of the CF data type.
        ///
        ///     Definition: This data type transmits codes and the formatted text associated with the code. This
        ///     data type can be used to transmit for the first time the formatted text for the canned text portion of
        ///     a report, for example, a standard radiological description for a normal chest X-ray. The receiving
        ///     system can store this information and in subsequent messages only the identifier need be sent.
        ///     Another potential use of this data type is transmitting master file records that contain formatted
        ///     text. This data type has six components as follows:
        ///     The components, primary and alternate, are defined exactly as in the CE data type with the
        ///     exception of the second and fifth components, which are of the formatted text data type.
        ///
        ///     Example:
        ///         OBX||CF|71020^CXR^99CPMC||79989^\H\Description:\N\\.sp\\ti+4\Heart is not
        ///             enlarged. There is no evidence of pneumonia, effusion, pneumothorax or
        ///             any masses. \.sp+3\\H\Impression:\N\\.sp\\.ti+4\Negative chest.^99CPMC
        ///
        /// ## 2A.3.7.1 Identifier (ST)
        ///     Definition: Sequence of characters (the code) that uniquely identifies the item being referenced by
        ///     the <text>. Different coding schemes will have different elements here.
        ///
        /// ## 2A.3.7.2 Formatted Text (FT)
        ///     Definition: Name or description of the item in question with the addition of embedded formatting
        ///     instructions.
        ///
        /// ## 2A.3.7.3 Name of Coding System (ID)
        ///     Definition: Contains the name of the coding system employed.
        ///
        ///     Refer to HL7 Table 0396 - Coding System in Chapter 2C, Code Tables, for valid values.
        ///
        ///     As of v2.7 this component is required when CF.1 is populated and CF.14 is not populated. Both
        ///     CF.3 and CF.14 may be populated. Receivers should not identify a code based on its position
        ///     within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or position within
        ///     a repeating field. Instead, the receiver should always examine the coding system as specified in
        ///     CF.3 and/or CF.14, the Coding System component or the Coding System OID, for the tuple.
        ///
        /// ## 2A.3.7.4 Alternate Identifier (ST)
        ///     Definition: A sequence of characters that uniquely identifies an alternate code. Analogous to CF-1
        ///     Identifier.
        ///
        /// **Usage Notes:** The Alternate Identifier is used to represent the local or user seen code as described.
        /// If present, it obeys the same rules of use and interpretation as described for component 1. If both
        /// are present, the identifiers in component 4 and component 1 should have exactly the same
        /// meaning, i.e., they should be exact synonyms.
        ///
        /// ## 2A.3.7.5 Alternate Formatted Text (FT)
        ///     Definition: Name or description of the alternate identifier in question with the addition of
        ///     embedded formatting instructions. Analogous to CF.2 Formatted Text.
        ///
        /// ## 2A.3.7.6 Name of Alternate Coding System (ID)
        ///     Definition: Contains the name of the coding system employed for the alternate identifier.
        ///     Analogous to CF.3 Name of Coding System.
        ///
        ///     Refer to HL7 Table 0396 - Coding System in Chapter 2C, Code Tables, for valid values.
        ///
        ///     As of v2.7 this component is required when CF.4 is populated and CF.17 is not populated. Both
        ///     CF.6 and CF.17 may be populated. Receivers should not identify a code based on its position
        ///     within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or position within
        ///     a repeating field. Instead, the receiver should always examine the coding ystem as specified in
        ///     CF.6 and/or CF.17, the Coding System component or the Coding System OID, for the tuple.
        ///
        /// ## 2A.3.7.7 Coding System Version ID (ST)
        ///     Definition: This component carries the version for the coding system identified by components 1-
        ///     3. If CF.3 is populated with a value other than HL7nnnn or is of table type user-defined, version
        ///     ID must be valued with an actual version ID. If CF.3 is populated with a value of HL7nnnn and
        ///     nnnn is of table type HL7, version ID may have an actual value or it may be absent. If version ID
        ///     is absent, it will be interpreted to have the same value as the HL7 version number in the message
        ///     header.
        ///
        /// ## 2A.3.7.8 Alternate Coding System Version ID (ST)
        ///     Definition: This component carries the version for the coding system identified by components 4-
        ///     6. Analogous To CF.7 Coding System Version ID.
        ///
        /// ## 2A.3.7.9 Original Text (ST)
        ///     Definition: The text as seen and/or selected by the user who entered the data. Original text can be
        ///     used in a structured user interface to capture what the user saw as a representation of the code on
        ///     the data input screen, or in a situation where the user dictates or directly enters text, it is the text
        ///     entered or uttered by the user. In a situation where the code is assigned sometime after the text was
        ///     entered, original text is the text or phrase used as the basis for assigning the code.
        ///
        /// ## 2A.3.7.10 Second Alternate Identifier (ST)
        ///     Definition: A sequence of characters that uniquely identifies an alternate code. Analogous to CF.1
        ///     Identifier.
        ///
        /// ## 2A.3.7.11 Second Alternate FormattedText (FT)
        ///     Definition: The descriptive or textual name of the Second Alternate Identifier. Analogous to CF.2
        ///     Formatted Text.
        ///
        /// ## 2A.3.7.12 Name of Second Alternate Coding System (ID)
        ///     Definition: Identifies the coding scheme being used in the Second Alternate Identifier component.
        ///     Analogous to CF. Name of Coding System.
        ///
        ///     This component is required when CF.10 is populated and CF.20 is not populated. Both CF.10 and
        ///     CF.20 may be populated. Receivers should not identify a code based on its position within the
        ///     tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or position within a repeating
        ///     field. Instead, the receiver should always examine the coding ystem as specified in CF.12 and/or
        ///     CF.20 the Coding System component or the Coding System OID for the tuple.
        ///
        /// ## 2A.3.7.13 Second Alternate Coding System Version ID (ST)
        ///     Definition: This component carries the version for the coding system identified by components 10-
        ///     12. Analogous To CF.7 Coding System Version ID.
        ///
        /// ## 2A.3.7.14 Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) for the coding system or
        ///     value set named in CNE.3. The value for this component is 2.16.840.1.113883.12.#### where
        ///     "####" is to be replaced by the HL7 table number in the case of an HL7 defined or user defined
        ///     table. For externally defined code systems the OID registered in the HL7 OID registry SHALL be
        ///     used.
        ///
        ///     This component is required when CF.1 is populated and CF.3 is not populated. Both CF.3 and
        ///     CF.14 may be populated.
        ///
        /// ## 2A.3.7.15 Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CWE.1 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used. A value set may or need not be present irrespective of other
        ///     fields.
        ///
        /// ## 2A.3.7.16 Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CF.15. The version is
        ///     a date. The date is the date/time that the value set being used was published.
        ///     Value set version ID is required if CF.15 is populated.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        ///
        /// ## 2A.3.7.17 Alternate Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) for the coding system or
        ///     value set named in CF.6. Analogous To CF.14 Coding System OID.
        ///
        ///     The value for this component is 2.16.840.1.113883.12.#### where "####" is to be replaced by the
        ///     HL7 table number in the case of an HL7 defined or user defined table. For externally defined code
        ///     systems the OID registered in the HL7 OID registry SHALL be used.
        ///
        ///     This component is required when CF.4 is populated and CF.6 is not populated. Both CF.6 and
        ///     CF.17 may be populated.
        ///
        /// ## 2A.3.7.18 Alternate Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CF.4 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        /// A value set may or need not be present irrespective of other fields.
        ///
        /// ## 2A.3.7.19 Alternate Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CF.18. The version is
        ///     a date. The date is the date/time that the value set being used was published.
        ///     Value set version ID is required if CF.18 is populated.
        ///
        /// ## 2A.3.7.20 Second Alternate Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) for the coding system or
        ///     value set named in CF.12. Analogous To CF.14 Coding System OID.
        ///
        ///     The value for this component is 2.16.840.1.113883.12.#### where "####" is to be replaced by the
        ///     HL7 table number in the case of an HL7 defined or user defined table. For externally defined code
        ///     systems the OID registered in the HL7 OID registry SHALL be used.
        ///
        ///     This component is required when CF.10 is populated and CF.12 is not populated. Both CF.12 and
        ///     CF.20 may be populated.
        ///
        /// ## 2A.3.7.21 Second Alternate Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CF.10 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        ///
        ///     A value set may or need not be present irrespective of other fields.
        ///
        /// ## 2A.3.7.22 Second Alternate Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CF.21. The version is
        ///     a date. The date is the date/time that the value set being used was published.
        ///     Value set version ID is required if CF.21 is populated.
        CF,
        ///
        /// # 2A.3.8 CNE – coded with no exceptions
        ///
        /// As of v2.7 a third tuple, formerly known as triplet, has been added to the CNE data type.
        /// Additionally, 3 new components were added to each tuple such that each tuple now has a total of 7
        /// components. The Original Text component applies to the CNE as a whole.
        ///
        /// **Note:** The Vocabulary TC is the steward of the CNE data type.
        ///
        /// Definition: Specifies a coded element and its associated detail. The CNE data type is used when a
        /// required or mandatory coded field is needed. The specified HL7 table or imported or externally
        /// defined coding system must be used and may not be extended with local values. Text may not
        /// replace the code. A CNE field must have an HL7 defined or external table associated with it. A
        /// CNE field may be context sensitive such that a choice of explicit coding systems might be
        /// designated. This allows for realm and other types of specificity. Every effort will be made to
        /// enumerate the valid coding system(s) to be specified in the 3rd component, however, the standards
        /// body realizes that this is impossible to fully enumerate.
        ///
        /// **Note:** The presence of two sets of equivalent codes in this data type is semantically different from a
        /// repetition of a CNE-type field. With repetition, several distinct codes (with distinct meanings) may be
        /// transmitted.
        ///
        ///     Example 1: The drug must be coded and must be taken from the specified coding system. The
        ///     coding system is an external coding system. Example is derived from FT1-26.
        ///         |0006-0106-58^Prinivil 10mg oral tablet^NDC|
        ///
        ///     Example 2: Consent mode must be coded and must be taken from the specified coding system.
        ///     The coding system is an HL7 code table. Example is taken from CON-10.
        ///         |V^Verbal^HL70497^^^^2.8|
        ///
        /// ## 2A.3.8.1 Identifier (ST)
        ///     Definition: The first component contains the sequence of characters (the code) that uniquely
        ///     identifies the item being referenced by the CNE.2. Different coding schemes will have different
        ///     elements here.
        ///
        /// **Usage Note:** The identifier is required and must be a valid code.
        ///
        /// ## 2A.3.8.2 Text (ST)
        ///     Definition: The second component contains the descriptive or textual name of the identifier, e.g.,
        ///     myocardial infarction or X-ray impression. This is the corresponding text assigned by the coding
        ///     system to the identifier.
        ///
        /// **Usage Note:** Text description of code is optional but its use should be encouraged since it makes
        /// messages easier to review for accuracy, especially during interface testing and debugging.
        ///
        /// ## 2A.3.8.3 Name of Coding System (ID)
        ///     Definition: The third component contains the code for the name of the coding system from which
        ///     the value in CNE.1 is obtained. Refer to HL7 Table 0396 - Coding Systems in Chapter 2C, Code
        ///     Tables, for valid values. Each coding system is assigned a unique identifier.
        ///
        ///     As of v2.7 this component is required when CNE.1 is populated and CNE.14 is not populated.
        ///     Both CNE.3 and CNE.14 may be populated. Receivers should not identify a code based on its
        ///     position within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or
        ///     position within a repeating field. Instead, the receiver should always examine the codingSystem as
        ///     specified in CNE.3 and/or CNE.14 the Coding System component or the Coding System OID for
        ///     the tuple.The combination of the identifier and name of coding system represent a unique
        ///     concept for a data item.
        ///
        ///     Some organizations that publish code sets author more than one. The coding system, then, to be
        ///     unique, is a concatenation of the name of the coding authority organization and the name of its
        ///     code set or table. When an HL7 table is used for a CNE data type, the name of coding system
        ///     component is defined as HL7nnnn where nnnn is the HL7 table number. Similarly, ISO tables
        ///     will be named ISOnnnn, where nnnn is the ISO table number.
        ///
        /// **Usage Note: The following statement is retained for backward compatibility as of v2.7.** Best
        /// practice would recommend that this component always be populated. However, if the field
        /// narrative explicitly states "Refer to HL7 Table nnnn for valid values”, and, if the sending and
        /// receiving systems are HL7 compliant, the coding system would be known from the standard. This
        /// would be similar to a field with an ID data type, except that there is a second triplet in which to
        /// express an alternate code.
        ///
        /// ## 2A.3.8.4 Alternate Identifier (ST)
        ///     Definition: A sequence of characters that uniquely identifies an alternate code. Analogous to
        ///     CNE.1 Identifier.
        ///
        /// **Usage Notes:** The Alternate Identifier is used to represent the local or user seen code as described.
        /// If present, it obeys the same rules of use and interpretation as described for component 1. If both
        /// are present, the identifiers in component 4 and component 1 should have exactly the same
        /// meaning, i.e., they should be exact synonyms.
        ///
        /// ## 2A.3.8.5 Alternate Text (ST)
        ///     Definition: The descriptive or textual name of the alternate identifier. Analogous to CNE.2 Text.
        ///
        /// **Usage Notes:** If present, CNE.5 obeys the same rules of use and interpretation as described for
        /// CNE.2.
        ///
        /// ## 2A.3.8.6 Name of Alternate Coding System (ID)
        ///     Definition: Identifies the coding scheme being used in the alternate identifier component.
        ///     Analogous to CNE.3 Name of Coding System. Refer to HL7 Table 0396 - Coding Systems in
        ///     Chapter 2C, Code Tables, for valid values.
        ///     As of v2.7 this component is required when CNE.4 is populated and CNE.17 is not populated.
        ///     Both CNE.6 and CNE.17 may be populated. Receivers should not identify a code based on its
        ///     position within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or
        ///     position within a repeating field. Instead, the receiver should always examine the codingSystem as
        ///     specified in CNE.6 and/or CNE.17, the "Coding System" component or the "Coding System OID",
        ///     for the tuple.
        ///
        /// **Usage Notes:** If present, CNE.6 obeys the same rules of use and interpretation as described for
        /// CNE.3.
        ///
        /// ## 2A.3.8.7 Coding System Version ID (ST)
        ///     Definition: the version ID for the coding system identified by CNE.3. It belongs conceptually to
        ///     components 1-3 and appears here only for reasons of backward compatibility.
        ///
        /// **Usage Note:** If the coding system is any system other than an "HL7 coding system," version ID
        /// must be valued with an actual version ID. If the coding system is "HL7 coding system," version
        /// ID may have an actual value or it may be absent. If version ID is absent, it will be interpreted to
        /// have the same value as the HL7 version number in the message header. Text description of code is
        /// optional but its use should be encouraged since it makes messages easier to review for accuracy,
        /// especially during interface testing and debugging.
        ///
        /// ## 2A.3.8.8 Alternate Coding System Version ID (ST)
        ///     Definition: the version ID for the coding system identified by CNE.6. It belongs conceptually to
        ///     the group of Alternate components (see note 2.A.1) and appears here only for reasons of backward
        ///     compatibility. Analogous to CNE.7 Coding System Version ID.
        ///
        /// **Usage Notes:** If present, CNE.8 obeys the same rules of use and interpretation as described for
        /// CNE.7.
        ///
        /// ## 2A.3.8.9 Original Text (ST)
        ///     Definition: The text as seen and/or selected by the user who entered the data. Original text can be
        ///     used in a structured user interface to capture what the user saw as a representation of the code on
        ///     the data input screen, or in a situation where the user dictates or directly enters text, it is the text
        ///     entered or uttered by the user. In a situation where the code is assigned sometime after the text was
        ///     entered, original text is the text or phrase used as the basis for assigning the code.
        ///
        /// ## 2A.3.8.10 Second Alternate Identifier (ST)
        ///     Definition: A sequence of characters that uniquely identifies a second alternate code. Analogous to
        ///     CN-1 Identifier.
        ///
        /// ## 2A.3.8.11 Second Alternate Text (ST)
        ///     Definition: The descriptive or textual name of the Second Alternate Identifier. Analogous to
        ///     CNE.2 Text.
        ///
        /// ## 2A.3.8.12 Name of Second Alternate Coding System (ID)
        ///     Definition: Identifies the coding scheme being used in the Second Alternate Identifier component.
        ///     Analogous to CNE.3 Name of Coding System. Refer to HL7 Table 0396 - Coding Systems in
        ///     Chapter 2C, Code Tables, for valid values.
        ///
        ///     As of v2.7 this component is required when CNE.10 is populated and CNE.20 is not populated.
        ///     Both CNE.12 and CNE.20 may be populated. Receivers should not identify a code based on its
        ///     position within the tuples (Identifier, Alternate Identifier, or Second Alternate Identifier) or
        ///     position within a repeating field. Instead, the receiver should always examine the codingSystem as
        ///     specified in CNE.12 and/or CNE.20, the "Coding System" component or the "Coding System
        ///     OID", for the tuple.
        ///
        /// ## 2A.3.8.13 Second Alternate Coding System Version ID (ST)
        ///     Definition: This component carries the version for the coding system identified by components 10-
        ///     12. Analogous to CNE.7 Coding System Version ID.
        ///
        /// ## 2A.3.8.14 Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) for the coding system or
        ///     value set named in CNE.3. The value for this component is 2.16.840.1.113883.12.#### where
        ///     "####" is to be replaced by the HL7 table number in the case of an HL7 defined or user defined
        ///     table. For externally defined code systems the OID registered in the HL7 OID registry SHALL be
        ///     used.
        ///
        ///     This component is required when CNE.1 is populated and CNE.3 is not populated. Both CNE.3
        ///     and CNE.14 may be populated.
        ///
        /// ## 2A.3.8.15 Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CNE.1 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used.
        ///
        ///     A value set may or need not be present irrespective of other fields.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        ///
        /// ## 2A.3.8.16 Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CNE.15. The version
        ///     is a date. The date is the date/time that the value set being used was published.
        ///     Value set version ID is required if CNE.15 is populated.
        ///
        /// ## 2A.3.8.17 Alternate Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) for the coding system or
        ///     value set named in CNE.6. Analogous to CNE.14 OID for Coding System.
        ///
        ///     The value for this component is 2.16.840.1.113883.12.#### where "####" is to be replaced by the
        ///     HL7 table number in the case of an HL7 defined or user defined table. For externally defined code
        ///     systems the OID registered in the HL7 OID registry SHALL be used.
        ///
        ///     This component is required when CNE.4 is populated and CNE.6 is not populated. Both CNE.6
        ///     and CNE.17 may be populated.
        ///
        /// ## 2A.3.8.18 Alternate Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CNE.4 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        /// A value set may or need not be present irrespective of other fields.
        ///
        /// ## 2A.3.8.19 Alternate Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CNE.18. The version
        ///     is a date. The date is the date/time that the value set being used was published.
        ///     Value set version ID is required if CNE.18 is populated.
        ///
        /// ## 2A.3.8.20 Second Alternate Coding System OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) from which the value in
        ///     CNE.12 is obtained. The value for this component is 2.16.840.1.113883.12.#### where "####" is
        ///     to be replaced by the HL7 table number in the case of an HL7 defined or user defined table. For
        ///     externally defined numbers, the OID registered in the HL7 OID registry should be used.
        ///
        ///     This component is required when CNE.4 is populated and neither CNE.6 nor CNE.18 is populated.
        ///     In short either the CNE.6 or the CNE.14 or CNE.17 must be populated when CNE.4 is populated.
        ///
        /// ## 2A.3.8.21 Second Alternate Value Set OID (ST)
        ///     Definition: This component contains the ISO Object Identifier (OID) to allow identification of the
        ///     value set from which the value in CNE.10 is obtained. The value for this component is
        ///     2.16.840.1.113883.12.#### where "####" is to be replaced by the HL7 table number in the case of
        ///     an HL7 defined or user defined table. For externally defined value sets, the OID registered in the
        ///     HL7 OID registry SHALL be used. A value set may or need not be present irrespective of other
        ///     fields.
        ///
        /// **Note:** If a code is provided, the meaning of the code must come from the definition of the code in the code
        /// system. The meaning of the code SHALL NOT depend on the value set. Applications SHALL NOT be
        /// required to interpret the code in light of the valueSet, and they SHALL NOT reject an instance because of
        /// the presence or absence of any or a particular value set/ value set version ID.
        ///
        /// ## 2A.3.8.22 Second Alternate Value Set Version ID (DTM)
        ///     Definition: This component carries the version for the value set identified by CNE.21. The version
        ///     is a date. The date is the date/time that the value set being used was published.
        ///
        ///     Value set version ID is required if CNE.21 is populated.
        ///
        CNE,
        ///
        /// # 2A.3.9 CNN - composite ID number and name simplified
        ///
        /// **Attention: Retained for backward compatibility only in version 2.6. Fields associated with this
        /// data type have been replaced by the ROL segment.**
        ///
        ///     Definition: Specifies a person using both an identifier and the person’s name. Retained for
        ///     backward compatibility only as of v2.6.
        ///
        /// **Note:** Restores the original data type CN as was initially implementable in the CM used in sections
        /// 4.5.3.32 and 7.4.1.32 - (OBR-32), 4.5.3.33 and 7.4.1.33 - ( OBR-33), 4.5.3.34 and 7.4.1.34 - ( OBR-34),
        /// 4.5.3.35 and 7.4.1.35 - (OBR-35). Components 7 and 8, however, have been promoted to data type IS to be
        /// consistent with current practice without violating backward compatibility.
        ///
        /// ## 2A.3.9.1 ID Number (ST)
        ///     Coded ID according to a user-defined table. If the first component is present, either CNN.8 or
        ///     CNN.9, or both CNN.10 and CNN.11, must be valued.
        ///
        /// ## 2A.3.9.2 Family Name (ST)
        ///     This component contains the person's family name in a string format.
        ///
        /// ## 2A.3.9.3 Given Name (ST)
        ///     Used to specify a first name.
        ///
        /// ## 2A.3.9.4Second and Further Given Names or Initials Thereof (ST)
        ///
        /// ## 2A.3.9.5Suffix (ST)
        ///     Used to specify a name suffix (e.g., Jr. or III).
        ///
        /// ## 2A.3.9.6 Prefix (ST)
        ///     Used to specify a name prefix (e.g., Dr.).
        ///
        /// ## 2A.3.9.7 Degree (IS)
        ///     Used to specify an educational degree (e.g., MD). Refer to User-defined Table 0360 –
        ///     Degree/license/certificate in Chapter 2C, Code Tables, for suggested values.
        ///
        /// ## 2A.3.9.8 Source Table (IS)
        ///     Refer to User-defined Table 0297 - CN ID source in Chapter 2C, Code Tables, for suggested
        ///     values. Used to delineate the first component. If component 1 is valued, either CNN.8 or CNN.9,
        ///     or both CNN.10 and CNN.11, must be valued.
        ///
        /// ## 2A.3.9.9 Assigning Authority - Namespace ID (IS)
        ///     See section, 2.A.14.4, "Assigning Authority (HD)" for definition. Refer to User-defined Table 0363
        ///     – Assigning Authority in Chapter 2C, Code Tables, for suggested values. Assigning Authority is
        ///     normally expressed as an HD data type, but has been flattened to 3 components here (CNN.9,
        ///     CNN.10 and CNN.11) in this data type so that it may be fully expressed. Also note that if
        ///     additional components are added to the HD data type in the future, adjustment will need to be
        ///     made accordingly to this data type.
        ///
        ///     If component 1 is valued, either CNN.8 or CNN.9, or both CNN.10 and CNN.11, must be valued.
        ///
        /// ## 2A.3.9.10 Assigning Authority - Universal ID (ST)
        ///     See section, 2.A.14.4, "Assigning Authority (HD)" for definition.
        ///     If CNN.11 is valued, this component must be valued. If component 1 is valued, either CNN.8 or
        ///     CNN.9, or both CNN.10 and CNN.11, must be valued.
        ///
        /// ## 2A.3.9.11 Assigning Authority - Universal ID Type (ID)
        ///     See section, 2.A.14.4, "Assigning Authority (HD)" for definition. If this component is a known
        ///     UID refer to HL7 Table 0301 - Universal ID type in Chapter 2C, Code Tables, for valid values.
        ///     If CNN.10 is valued, this component must be valued. If component 1 is valued, either CNN.8 or
        ///     CNN.9, or both CNN.10 and CNN.11, must be valued.
        ///
        CNN,
        ///
        /// # 2A.3.10 CP - composite price
        ///
        /// This data type is often used to define a repeating field within a given segment.
        ///
        /// Example:
        ///
        ///     |100.00&USD^UP^0^9^min^P~50.00&USD^UP^10^59^min^P~10.00&USD^UP^60^999^P~50
        ///     .00&USD^AP~200.00
        ///
        /// ## 2A.3.10.1 Price (MO)
        ///     Definition: The only required component; usually containing a decimal point.
        ///
        /// **Note:** Each component of the MO data type (Section 2.A.41, "MO - money") is a subcomponent here.
        ///
        /// ## 2A.3.10.2 Price Type (ID)
        ///     Definition: A coded value, data type ID. Refer to HL7 Table 0205 – Price Type in Chapter 2C,
        ///     Code Tables, for valid values.
        ///
        /// ## 2A.3.10.3 From Value (NM)
        ///     Definition: The number specifying the lower limit or boundary of the range. This component,
        ///     together with the CP.4 component, specifies the "price range". The range can be defined as either
        ///     time or quantity. For example, the range can indicate that the first 10 minutes of the procedure has
        ///     one price. Another repetition of the data type can use the range to specify that the following 10 to
        ///     60 minutes of the procedure is charged at another price per; a final repetition can specify that the
        ///     final 60 to N minutes of the procedure at a third price.
        ///
        /// **Note:** If the CP.2 Price Type component is TP, both CP.3 and CP.4 may be null.
        ///
        /// ## 2A.3.10.4 To Value (NM)
        ///     Definition: The number specifying the high limit or boundary of the range.
        ///
        /// ## 2A.3.10.5 Range Units (CWE)
        /// Definition: This component describes the units associated with the range, e.g., seconds, minutes,
        /// hours, days, quantity (i.e., count). As of v2.7 the Externally-defined Unified Code for Units of
        /// Measure (UCUM) case sensitive code is the required code for units of measure. Refer to the
        /// externally-defined table ["Unified Code for Units of Measure" (UCUM)](http://aurora.rg.iupui.edu/UCUM)
        /// for valid values. Local codes may be transmitted in
        /// addition to UCUM codes.
        ///
        /// This component is required if CP.3 From Value and/or CP.4 To Value are present.
        ///
        /// ## 2A.3.10.6 Range Type (ID)
        ///     Definition: Refer to HL7 Table 0298 - CP Range Type for valid values.
        ///
        CP,
        ///
        /// # 2A.3.11 CQ - composite quantity with units
        ///
        /// **Note:** CQ cannot be legally expressed when embedded within another data type. Its use is constrained to a
        /// segment field.
        ///
        /// ### Examples:
        ///     |123.7^kg|          kilograms is an ISO unit
        ///     |150^lb&&ANSI+|     weight in pounds is a customary US unit defined within ANSI+.
        ///
        /// ## 2A.3.11.1 Quantity (NM)
        ///     Definition: This component specifies the numeric quantity or amount of an entity.
        ///
        /// ## 2A.3.11.2 Units (CWE)
        /// Definition: This component species the units in which the quantity is expressed. As of v2.7 the
        /// externally-defined Unified Code for Units of Measure (UCUM) case sensitive code is the required
        /// code for units of measure. Refer to the external table
        /// ["Unified Code for Units of Measure" (UCUM)](http://aurora.rg.iupui.edu/UCUM) for valid values.
        /// Local codes may be transmitted in addition to UCUM codes.
        ///
        /// Refer to user-defined Table 0794 - Units in Chaper 2C, Code Tables, for valid values.
        ///
        CQ,
        ///
        ///
        ///
        CSU,
        CWE,
        MO,
        NR,
        WVI,
        WVS,
    }

    #[derive(Debug)]
    pub enum V2ComponentType {
        Primitive(V2PrimitiveType),
        Complex(V2ComplexType),
    }

    #[derive(Debug)]
    pub struct V2ComponentTypeDescriptor {
        pub name: &'static str,
        pub description: &'static str,
        pub data_type: V2ComponentType,
        pub max_input_len: u32,
        pub seq: u16,
        pub valid_table: u16,
        pub optionality: Optionality,
        pub truncate: bool,
    }

    impl V2ComponentTypeDescriptor {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            data_type: V2ComponentType,
            max_input_len: u32,
            seq: u16,
            valid_table: u16,
            optionality: Optionality,
            truncate: bool,
        ) -> V2ComponentTypeDescriptor {
            V2ComponentTypeDescriptor {
                name,
                description,
                data_type,
                max_input_len,
                seq,
                valid_table,
                optionality,
                truncate,
            }
        }
    }

    pub type V2ComponentDescriptor = [&'static V2ComponentTypeDescriptor];
    pub type V2FieldDescriptors = Map<&'static str, &'static V2ComponentDescriptor>;

    ///
    /// Generates instance of V2ComponentDescriptor which defines how we should cast a field.
    ///
    /// ## Arguments
    /// * `name` - String representing the field name.
    /// * `description` - String describing the component as given in the data type table.
    /// * `data_type` - Appropriate [`V2PrimitiveType`] enumerator item describing the type we should
    ///     target when casting the field
    /// * `max_input_len` - Length to truncate value of field if `truncate` is True
    /// * `seq` - Number of field/sequence in segment.
    /// * `valid_table` - Validation table used for additional validation of input. It's a number now,
    ///     but may be changed to an enumerator in the future.
    /// * `optionality` - Option allowing marking the field as required. If a required field is missing,
    ///     emit error. If a field is flagged as conditional, we expect an Option passed to be passed
    ///     with None or a tuple of conditions.
    /// * `truncate` - Boolean flag for marking the component as one that needs to be truncated to `max_input_len`.
    ///
    #[macro_export]
    macro_rules! v2_component_descriptor {
        (
            $name:expr,
            $description:expr,
            $data_type:expr,
            $max_input_len:expr,
            $seq:expr,
            $valid_table:expr,
            $optionality:expr,
            $truncate:expr ) => {{
            &V2ComponentTypeDescriptor::new(
                $name,
                $description,
                $data_type,
                $max_input_len,
                $seq,
                $valid_table,
                $optionality,
                $truncate,
            )
        }};
    }

    pub static V2_FIELD_DESCRIPTORS: V2FieldDescriptors = phf_map! {
        "AD" => &[
            v2_component_descriptor!("street_address", "Street Address", V2ComponentType::Primitive(V2PrimitiveType::ST), 120, 1, 0, Optionality::O, true),
            v2_component_descriptor!("second_address", "Other Designation", V2ComponentType::Primitive(V2PrimitiveType::ST), 120, 2, 0, Optionality::O, true),
            v2_component_descriptor!("city", "City", V2ComponentType::Primitive(V2PrimitiveType::ST), 50, 3, 0, Optionality::O, true),
            v2_component_descriptor!("state", "State or Province", V2ComponentType::Primitive(V2PrimitiveType::ST), 50, 4, 0, Optionality::O, true),
            v2_component_descriptor!("zip", "Zip or Postal Code", V2ComponentType::Primitive(V2PrimitiveType::ST), 12, 5, 0, Optionality::O, false),
            v2_component_descriptor!("country", "Country", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 6, 399, Optionality::O, false),
            v2_component_descriptor!("address_type", "Address Type", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 7, 190, Optionality::O, false),
            v2_component_descriptor!("county", "Other Geographic Designation", V2ComponentType::Primitive(V2PrimitiveType::ST), 50, 8, 0, Optionality::O, true)
        ],
        "AUI" => &[
            v2_component_descriptor!("auth_number", "Authorization Number", V2ComponentType::Primitive(V2PrimitiveType::ST), 30, 1, 0, Optionality::O, false),
            v2_component_descriptor!("date", "Date", V2ComponentType::Primitive(V2PrimitiveType::Date), 0, 2, 0, Optionality::O, false),
            v2_component_descriptor!("source", "Source", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 3, 0, Optionality::O, true)
        ],
        "CCD" => &[
            v2_component_descriptor!("event", "Invocation Event", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 1, 0, Optionality::R, false),
            v2_component_descriptor!("date", "Date/time", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 0, 2, 100, Optionality::O, false)
        ],
        "CCP" => &[
            v2_component_descriptor!("cc_factor", "Channel Calibration Sensitivity Correction Factor", V2ComponentType::Primitive(V2PrimitiveType::NM), 6, 1, 0, Optionality::O, true),
            v2_component_descriptor!("cc_baseline", "Channel Calibration Baseline", V2ComponentType::Primitive(V2PrimitiveType::NM), 6, 2, 0, Optionality::O, true),
            v2_component_descriptor!("cc_time_skew", "Channel Calibration Time Skew", V2ComponentType::Primitive(V2PrimitiveType::NM), 6, 3, 0, Optionality::O, true)
        ],
        "CD" => &[
            v2_component_descriptor!("channel_id", "Channel Identifier", V2ComponentType::Complex(V2ComplexType::WVI), 0, 1, 0, Optionality::O, false),
            v2_component_descriptor!("waveform_source", "Waveform Source", V2ComponentType::Complex(V2ComplexType::WVS), 0, 2, 0, Optionality::O, false),
            v2_component_descriptor!("channel_sensitivity_units", "Channel Sensitivity and Units", V2ComponentType::Complex(V2ComplexType::CSU), 0, 3, 0, Optionality::O, false),
            v2_component_descriptor!("channel_calibration_parameters", "Channel Calibration Parameters", V2ComponentType::Complex(V2ComplexType::CCP), 0, 4, 0, Optionality::O, false),
            v2_component_descriptor!("channel_sampling_frequency", "Channel Sampling Frequency", V2ComponentType::Primitive(V2PrimitiveType::NM), 6, 5, 0, Optionality::O, true),
            v2_component_descriptor!("min_max_values", "Minimum and Maximum Data Values", V2ComponentType::Complex(V2ComplexType::NR), 0, 6, 0, Optionality::O, false)
        ],
        "CE" => &[  ],
        "CF" => &[
            v2_component_descriptor!("id", "Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 1, 0, Optionality::O, false),
            v2_component_descriptor!("formatted_text", "Formatted Text", V2ComponentType::Primitive(V2PrimitiveType::FT), 0, 2, 0, Optionality::O, false),
            v2_component_descriptor!("coding_system", "Name of Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 3, 396, Optionality::C(CONDITION_CF1), false),
            v2_component_descriptor!("alt_id", "Alternate Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 4, 0, Optionality::O, false),
            v2_component_descriptor!("alt_formatted_text", "Alternate Formatted Text", V2ComponentType::Primitive(V2PrimitiveType::FT), 0, 5, 0, Optionality::O, false),
            v2_component_descriptor!("alt_coding_system", "Name of Alternate Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 6, 396, Optionality::C(CONDITION_CF2), false),
            v2_component_descriptor!("version_id", "Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 7, 0, Optionality::C(CONDITION_CF3), false),
            v2_component_descriptor!("alt_version_id", "Alternate Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 8, 0, Optionality::O, false),
            v2_component_descriptor!("original_text", "Original Text", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 9, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_id", "Second Alternate Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 10, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_formatted_text", "Second Alternate Formatted Text", V2ComponentType::Primitive(V2PrimitiveType::FT), 199, 11, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_coding_system", "Name of Second Alternate Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 12, 396, Optionality::O, false),
            v2_component_descriptor!("second_alt_version_id", "Second Alternate Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 13, 0, Optionality::C(CONDITION_CF4), false),
            v2_component_descriptor!("coding_system_oid", "Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 14, 0, Optionality::C(CONDITION_CF5), false),
            v2_component_descriptor!("valueset_oid", "Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 15, 0, Optionality::O, false),
            v2_component_descriptor!("valueset_version_id", "Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 16, 0, Optionality::C(CONDITION_CF6), false),
            v2_component_descriptor!("alt_coding_system_oid", "Alternate Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 17, 0, Optionality::C(CONDITION_CF7), false),
            v2_component_descriptor!("alt_valueset_oid", "Alternate Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 18, 0, Optionality::O, false),
            v2_component_descriptor!("alt_valueset_version_id", "Alternate Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 19, 0, Optionality::C(CONDITION_CF8), false),
            v2_component_descriptor!("second_alt_coding_system_oid", "Second Alternate Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 20, 0, Optionality::C(CONDITION_CF9), false),
            v2_component_descriptor!("second_alt_valueset_oid", "Second Alternate Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 21, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_valueset_version_id", "Second Alternate Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 22, 0, Optionality::C(CONDITION_CF10), false)
        ],
        "CNE" => &[
            v2_component_descriptor!("id", "Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 1, 0, Optionality::R, false),
            v2_component_descriptor!("text", "Text", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 2, 0, Optionality::O, true),
            v2_component_descriptor!("coding_system", "Name of Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 3, 396, Optionality::O, false),
            v2_component_descriptor!("alt_id", "Alternate Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 4, 0, Optionality::O, false),
            v2_component_descriptor!("alt_text", "Alternate Text", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 5, 0, Optionality::O, true),
            v2_component_descriptor!("alt_coding_system", "Name of Alternate Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 6, 396, Optionality::O, false),
            v2_component_descriptor!("version_id", "Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 7, 0, Optionality::C(CONDITION_CNE1), false),
            v2_component_descriptor!("alt_version_id", "Alternate Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 8, 0, Optionality::O, false),
            v2_component_descriptor!("original_text", "Original Text", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 9, 0, Optionality::O, true),
            v2_component_descriptor!("second_alt_id", "Second Alternate Identifier", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 10, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_text", "Second Alternate Text", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 11, 0, Optionality::O, true),
            v2_component_descriptor!("second_alt_coding_system", "Name of Second Alternate Coding System", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 12, 396, Optionality::O, false),
            v2_component_descriptor!("second_alt_version_id", "Second Alternate Coding System Version ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 10, 13, 0, Optionality::C(CONDITION_CNE2), false),
            v2_component_descriptor!("coding_system_oid", "Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 14, 0, Optionality::C(CONDITION_CNE3), false),
            v2_component_descriptor!("valueset_oid", "Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 15, 0, Optionality::O, false),
            v2_component_descriptor!("valueset_version_id", "Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 16, 0, Optionality::C(CONDITION_CNE4), false),
            v2_component_descriptor!("alt_coding_system_oid", "Alternate Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 17, 0, Optionality::C(CONDITION_CNE5), false),
            v2_component_descriptor!("alt_valueset_oid", "Alternate Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 18, 0, Optionality::O, false),
            v2_component_descriptor!("alt_valueset_version_id", "Alternate Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 19, 0, Optionality::C(CONDITION_CNE6), false),
            v2_component_descriptor!("second_alt_coding_system_oid", "Second Alternate Coding System OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 20, 0, Optionality::C(CONDITION_CNE7), false),
            v2_component_descriptor!("second_alt_valueset_oid", "Second Alternate Value Set OID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 21, 0, Optionality::O, false),
            v2_component_descriptor!("second_alt_valueset_version_id", "Second Alternate Value Set Version ID", V2ComponentType::Primitive(V2PrimitiveType::DateTime), 8, 22, 0, Optionality::C(CONDITION_CNE8), false)
        ],
        "CNN" => &[
            v2_component_descriptor!("id", "ID Number", V2ComponentType::Primitive(V2PrimitiveType::ST), 15, 1, 0, Optionality::O, false),
            v2_component_descriptor!("family_name", "Family Name", V2ComponentType::Primitive(V2PrimitiveType::ST), 50, 2, 0, Optionality::O, true),
            v2_component_descriptor!("given_name", "Given Name", V2ComponentType::Primitive(V2PrimitiveType::ST), 30, 3, 0, Optionality::O, true),
            v2_component_descriptor!("second_given_name", "Second and Further Given Names or Initials Thereof", V2ComponentType::Primitive(V2PrimitiveType::ST), 30, 4, 0, Optionality::O, true),
            v2_component_descriptor!("suffix", "Suffix (e.g. JR or III)", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 5, 0, Optionality::O, true),
            v2_component_descriptor!("prefix", "Prefix (e.g. DR)", V2ComponentType::Primitive(V2PrimitiveType::ST), 20, 6, 0, Optionality::O, true),
            v2_component_descriptor!("degree", "Degree (e.g. MD)", V2ComponentType::Primitive(V2PrimitiveType::IS), 6, 7, 360, Optionality::O, false),
            v2_component_descriptor!("source_table", "Source Table", V2ComponentType::Primitive(V2PrimitiveType::IS), 4, 8, 297, Optionality::C(CONDITION_CNN1), false),
            v2_component_descriptor!("aa_namespace_id", "Assigning Authority - Namespace ID", V2ComponentType::Primitive(V2PrimitiveType::IS), 20, 9, 363, Optionality::C(CONDITION_CNN2), false),
            v2_component_descriptor!("aa_universal_id", "Assigning Authority - Universal ID", V2ComponentType::Primitive(V2PrimitiveType::ST), 199, 10, 0, Optionality::C(CONDITION_CNN3), false),
            v2_component_descriptor!("aa_universal_id_type", "Assigning Authority - Universal ID Type", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 11, 301, Optionality::C(CONDITION_CNN4), false)
        ],
        "CP" => &[
            v2_component_descriptor!("price", "Price", V2ComponentType::Complex(V2ComplexType::MO), 0, 1, 0, Optionality::R, false),
            v2_component_descriptor!("price_type", "Price Type", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 2, 205, Optionality::O, false),
            v2_component_descriptor!("from_value", "From Value", V2ComponentType::Primitive(V2PrimitiveType::NM), 0, 3, 0, Optionality::O, false),
            v2_component_descriptor!("to_value", "To Value", V2ComponentType::Primitive(V2PrimitiveType::NM), 0, 4, 0, Optionality::O, false),
            v2_component_descriptor!("range_units", "Range Units", V2ComponentType::Complex(V2ComplexType::CWE), 0, 5, 0, Optionality::C(CONDITION_CP), false),
            v2_component_descriptor!("range_type", "Range Type", V2ComponentType::Primitive(V2PrimitiveType::ID), 0, 6, 298, Optionality::O, false)
        ],
        "CQ" => &[
            v2_component_descriptor!("quantity", "Quantity", V2ComponentType::Primitive(V2PrimitiveType::NM), 0, 1, 0, Optionality::O, false),
            v2_component_descriptor!("units", "Units", V2ComponentType::Complex(V2ComplexType::CWE), 0, 2, 794, Optionality::O, false)
        ],
        "CSU" => &[
            v2_component_descriptor!("quantity", "Quantity", V2ComponentType::Primitive(V2PrimitiveType::NM), 0, 1, 0, Optionality::O, false),
            v2_component_descriptor!("units", "Units", V2ComponentType::Complex(V2ComplexType::CWE), 0, 2, 794, Optionality::O, false)
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
            V2ComplexType::CE => "CE",
            V2ComplexType::CF => "CF",
            V2ComplexType::CNE => "CNE",
            V2ComplexType::CNN => "CNN",
            V2ComplexType::CP => "CP",
            V2ComplexType::CQ => "CQ",
            V2ComplexType::CSU => "CSU",
            _ => "Error",
        }
    }
}
