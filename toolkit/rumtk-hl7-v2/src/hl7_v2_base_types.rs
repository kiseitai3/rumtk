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

pub mod v2_base_types {
    use chrono::prelude::*;
    use rumtk_core::strings::{count_tokens_ignoring_pattern, decompose_dt_str, format_compact, ToCompactString};
    use rumtk_core::maths::generate_tenth_factor;
    use crate::hl7_v2_constants::{V2_DATETIME_MIRCRO_LENGTH, V2_DATETIME_THOUSAND_TICK};
    use rumtk_core::strings::{RUMString};

    ///
    /// Nothing fancier than a SSO string.
    /// Basic type used to derive other types for the standard implementation.
    ///
    pub type V2String = RUMString;
    ///
    /// 2A.3.76 ST - string data
    ///
    /// # Definition:
    ///     The String data type is used for text data when the appearance of text does not bear
    ///     meaning. This is true for formalized text, symbols and formal expressions, and all kinds of names
    ///     intended for machine processing (e.g., sorting, querying, indexing, etc.).
    ///
    ///     String data is left justified (i.e., no leading blank space) with trailing blanks optional, which may
    ///     be trimmed, and SHOULD be ignored on string compare operations for 2 values of type ST. Any
    ///     displayable (printable) characters are allowed based on the character set identified in MSH-18. For
    ///     the default ASCII character set this is hexadecimal values between 20 and 7E, inclusive, or
    ///     decimal values between 32 and 126, except the defined escape characters and defined delimiter
    ///     characters. For Unicode this is any code point with a Basic Type of Graphic, except the defined
    ///     escape characters and defined delimiter characters; see The Unicode Standard section 2.4
    ///     <http://www.unicode.org/versions/Unicode10.0.0/ch02.pdf> for details.
    ///
    /// ## Example 1:
    ///     A textual ST field:
    ///     |almost any data at all|
    /// ## Example 2:
    ///     URL encoded in an ST component:
    ///         ^http://www.pacs.poupon.edu/wado.jsp^
    /// ## Example 3:
    ///     ISO OID encoded in an ST subcomponent:
    ///         &2.16.840.1.113883.1.1&
    ///
    /// To include any HL7 delimiter character (except the segment terminator) within a string data field,
    /// use the appropriate HL7 escape sequence (see Section 2.7.1, "Formatting Codes”).
    ///
    ///     Minimum Length: Not specified for the type. May be specified in the context of use. Defaults to 1
    ///     Maximum Length: Not specified for the type. May be specified in the context of use
    ///
    /// ST has no inbuilt semantics – these are assigned where the ST is used. In each case where ST is
    /// used, minimum, maximum, and conformance lengths may be specified. Unless specified in the
    /// context of use, values of type ST may not be truncated.
    ///
    /// ## Usage note:
    ///     The ST data type is intended for short strings (e.g., less than 1000 characters). For longer
    ///     strings the TX or FT data types should be used (see Sections 2.A.79, “TX - text data” or 2.A.31, “FT -
    ///     formatted text data”).
    ///
    ///     Alternate character set note: ST - string data may also be used to express other character sets. See Section
    ///     2.15.9.18, "Character set," and Section 2.15.9.20, "Alternate character set handling" for details.
    ///
    pub type V2ST = V2String;
    ///
    /// It's ambiguous how to handle an ID other than keep it as a string and not really validate it. See Section 2A.3.35
    /// Per Section 2A.3.35
    ///
    /// ## Definition:
    ///
    ///     The value of such a field follows the formatting rules for an ST field except that it is
    ///     drawn from a table of legal values. There shall be an HL7 table number associated with ID data
    ///     types. An example of an ID field is OBR-25 Result Status. This data type should be used only for
    ///     HL7 tables (see Chapter 2C, section 2.C.1.2, "HL7 Tables"). The reverse is not true, since in some
    ///     circumstances it is more appropriate to use the CNE or CWE data type for HL7 tables.
    ///
    ///     The minimum and maximum lengths are specified in the context in which the ID data type is used.
    ///     The longest HL7 defined legal value is 15 characters, but there are a few circumstances where the
    ///     legal values are taken from code systems defined by other bodies (such as IANA mime types). In
    ///     these cases, a different conformance length may be specified where the ID data type is used. It is
    ///     never acceptable to truncate an ID value.
    ///
    pub type V2ID = V2String;
    ///
    /// Pretty much the same as the ID type so we are simply aliasing that type here.
    ///
    /// Per Section 2A.3.36
    ///
    ///     As of v2.7, the only approved use of the IS data type is in the HD.1, EI.2 and PL.6 plus a limited
    ///     number of fields where a determination could not readily be made as to whether the item is an
    ///     identifier or an actual coded item. Additionally, in accordance with chapter 2 rules, any field or
    ///     data type component marked as "Retained for backward compatibility" will retain any IS data
    ///     type.
    ///
    ///     The value of such a field follows the formatting rules for a ST field except that it is drawn from a
    ///     site-defined (or user-defined) table of legal values. There shall be an HL7 table number associated
    ///     with IS data types. An example of an IS field is the Event reason code defined in Chapter 3,
    ///     "Patient Administration", section 3.4.1.4, "Event reason code". This data type should be used only
    ///     for user-defined tables (see Chapter 2C, "Code Tables", section 2.C.1.1, "User-defined Tables").
    ///     The reverse is not true, since in some circumstances, it is more appropriate to use the CWE data
    ///     type for user-defined tables.
    ///
    ///     It is never acceptable to truncate an IS value.
    ///
    pub type V2IS = V2ID;
    ///
    /// Formatted Text type
    ///
    /// Alias of V2String since it is simply a string.
    ///
    /// Per Section 2A.3.31
    ///
    /// # Definition:
    ///
    ///     This data type is derived from the TX data type by allowing the addition of embedded
    ///     formatting instructions. These instructions are limited to those that are intrinsic and independent of
    ///     the circumstances under which the field is being used. The actual instructions and their
    ///     representation are described in section 2.7.6, “Usage and Examples of Formatted Text”. The FT
    ///     field is of arbitrary length (up to 64k) and may contain formatting commands enclosed in escape
    ///     characters.
    ///
    pub type V2FT = V2String;
    ///
    /// 2A.3.80 TX - text data
    ///
    /// # Definition:
    ///     String data meant for user display (on a terminal or printer). Such data would not
    ///     necessarily be left justified since leading spaces may contribute greatly to the clarity of the
    ///     presentation to the user. Because this type of data is intended for display, it may contain certain
    ///     escape character sequences designed to control the display. Escape sequence formatting is defined
    ///     in Section 2.7, "Use of escape sequences in text fields". Leading spaces should be included.
    ///     Trailing spaces should be removed.
    ///
    /// ## Example:
    ///  | leading spaces are allowed.|
    //
    /// Since TX data is intended for display purposes, the repeat delimiter, when used with a TX data
    /// field, implies a series of repeating lines to be displayed on a printer or terminal. Therefore, the
    /// repeat delimiters are regarded as paragraph terminators or hard carriage returns (e.g., they would
    /// display as though a CR/LF were inserted in the text (DOS type system) or as though a LF were
    /// inserted into the text (UNIX style system)).
    ///
    /// A receiving system would word-wrap the text between repeat delimiters in order to fit it into an
    /// arbitrarily sized display window but start any line beginning with a repeat delimiter on a new line.
    /// To include alternative character sets, use the appropriate escape sequence. See Chapter 2, section
    /// 2.14.9.18, "MSH-18 - Character Set" and section 2.14.9.20, "MSH-20 - Alternate Character Set
    /// Handling Scheme".
    ///
    /// This specification applies no limit to the length of the TX data type, either here where the data
    /// type is defined, or elsewhere where the data type is used. While there is no intrinsic reason to limit
    /// the length of this data type for semantic or syntactical reasons, it is expected that some sort of
    /// limitation will be imposed for technical reasons in implementations. HL7 recommends that
    /// implementation length limits be published in implementation profiles.
    ///
    pub type V2TX = V2String;
    ///
    /// Struct meant to be used when parsing a date or datetime encoded in a v2 component.
    ///
    /// Per Section 2A.3.22 DTM - date/time
    ///
    /// ## Definition:
    ///
    ///     Specifies a point in time using a 24-hour clock notation.
    ///
    ///     Minimum Length: 4
    ///     Maximum Length: 24
    ///
    ///     The number of characters populated (excluding the time zone specification) specifies the
    ///     precision.
    ///
    /// ## Format:
    ///     YYYY[MM[DD[HH[MM[SS[.S[S[S[S]]]]]]]]][+/-ZZZZ].
    ///
    /// ## Thus:
    ///
    ///     a) only the first four are used to specify a precision of "year"
    ///     b) the first six are used to specify a precision of "month"
    ///     c) the first eight are used to specify a precision of "day"
    ///     d) the first ten are used to specify a precision of "hour”
    ///     e) the first twelve are used to specify a precision of "minute”
    ///     f) the first fourteen are used to specify a precision of "second”
    ///     g) the first sixteen are used to specify a precision of "one tenth of a second”
    ///     h) the first nineteen are used to specify a precision of " one ten thousandths of a second”
    ///
    /// ## Example:
    ///
    ///     |199904| specifies April 1999.
    ///
    /// The time zone (+/-ZZZZ) is represented as +/-HHMM offset from Coordinated Universal Time (UTC)
    ///
    ///     •For implementations prior to V2.9 +0000 or -0000 both represent UTC (without offset).
    ///     •For implementations starting with V2.9
    ///         + use of the plus sign (+0000) represents the civil time zone offset is known to be zero,
    ///         + use of the minus sign (-0000) represents UTC (without offset)
    ///
    pub struct V2DateTime {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        offset: V2String
    }

    impl V2DateTime {
        pub fn new() -> V2DateTime {
            V2DateTime {
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
                microsecond: 0,
                offset: V2String::from("0"),
            }
        }

        pub fn from_utc_datetime(utc_dt: &DateTime<Utc>) -> V2DateTime {
            V2DateTime {
                year: utc_dt.year() as u16,
                month: utc_dt.month() as u8,
                day: utc_dt.day() as u8,
                hour: utc_dt.hour() as u8,
                minute: utc_dt.minute() as u8,
                second: utc_dt.second() as u8,
                microsecond: utc_dt.nanosecond() / (V2_DATETIME_THOUSAND_TICK as u32),
                offset: utc_dt.offset().to_compact_string(),
            }
        }

        /// Begin decomposing string into discrete components per HL7 DateTime format specs.
        /// See https://hl7-definition.caristix.com/v2/HL7v2.8/DataTypes/DTM
        pub fn from_v2_string(item: &V2String) -> V2DateTime {
            let dt_vec: Vec<&str> = item.split('.').collect();
            let mut offset_sign = "+";
            let mut ms_vec: Vec<&str> = dt_vec.last().unwrap().split(&offset_sign).collect();
            if count_tokens_ignoring_pattern(&ms_vec, &RUMString::from(" ")) < 2 {
                ms_vec = dt_vec.last().unwrap().split('-').collect();
                offset_sign = "-";
            }

            let (year, month, day, hour, minute, second) =
                decompose_dt_str(&RUMString::from(dt_vec[0]));

            // Now let's grab the two components of the vector and generate the microsecond and offset bits.
            let ms_string = ms_vec[0];
            let ms_string_len = ms_string.len();
            let microsecond = match ms_string_len {
                0 => 0,
                _ => ms_string.parse::<u32>().unwrap() *
                    generate_tenth_factor(
                        (V2_DATETIME_MIRCRO_LENGTH - (ms_string_len as u8)) as u32)
            };

            let offset: V2String = offset_sign.to_compact_string() + ms_vec[1];


            V2DateTime { year, month, day, hour, minute, second, microsecond, offset}
        }

        pub fn as_utc_string(&self) -> String {
            format!(
                "{year}-{month}-{day}T{hour}:{minute}:{second}.{microsecond}{offset}",
                year = self.year,
                month = self.month,
                day = self.day,
                hour = self.hour,
                minute = self.minute,
                second = self.second,
                microsecond = self.microsecond,
                offset = self.offset
            )
        }

        pub fn as_utc_datetime(&self) -> DateTime<Utc> {
            self.as_utc_string().parse().unwrap()
        }
        pub fn as_v2_date(&self) -> V2String {
            format_compact!("{:04}{:02}{:02}", &self.year, &self.month, &self.day)
        }
        pub fn as_v2_date_time(&self) -> V2String {
            format_compact!("{:04}{:02}{:02}{:02}{:02}{:02}.{:04}", &self.year, &self.month, &self.day, &self.hour, &self.minute, &self.second, &self.microsecond)
        }
    }
    ///
    /// We can just use the V2DateTime type to represent this type.
    ///
    /// Section 2A.3.21 DT - date
    ///
    pub type V2Date = V2DateTime;
    // TODO: Missing the TM time type.

    ///
    /// 2A.3.47 NM - numeric
    ///
    /// # Definition:
    ///
    ///     A number represented as a series of ASCII numeric characters consisting of an optional
    ///     leading sign (+ or -), the digits and an optional decimal point. In the absence of a sign, the number
    ///     is assumed to be positive. If there is no decimal point the number is assumed to be an integer.
    ///
    ///     Minimum Length: 1
    ///     Maximum Length: 16
    ///
    /// ## Examples:
    ///
    ///     |999|
    ///     |-123.792|
    ///     |0.1|
    ///
    /// Values of this data type shall contain at least one digit to the left of the decimal point. This means
    /// that 0.1 is a valid representation, while .1 is not. Leading zeros, or trailing zeros after a decimal
    /// point, are not significant. For example, the following two values with different representations,
    /// "01.20" and "1.2," are identical. Except for the optional leading sign (+ or -) and the optional
    /// decimal point (.), no non-numeric ASCII characters are allowed. Thus, the value <12 should be
    /// encoded as a structured numeric (SN) (preferred) or as a string (ST) (allowed, but not preferred)
    /// data type.
    ///
    /// The NM data type does not follow the normal truncation pattern, and the truncation character is
    /// never valid in the NM data type. Instead, the truncation behavior is based on the semantics of
    /// numbers.
    ///
    /// Values of type NM may always have leading zeros truncated. Note that HL7 recommends that
    /// leading zeros not be used. Unless NM is used to represent a monetary amount, implementations
    /// may truncate trailing zeros after the decimal point up to the first non-zero digit or the decimal
    /// point, which ever comes first. Any digits to the left of the decimal point may never be truncated
    /// (other than leading zeros).
    ///
    /// ## Example:
    ///
    ///     1.0200 may be truncated to 1.02, but not to 1.0.
    ///
    pub type NM = f64;
    ///
    /// 2A.3.70 SI - sequence ID
    ///
    /// # Definition:
    ///     A non-negative integer in the form of a NM field. The uses of this data type are
    ///     defined in the chapters defining the segments and messages in which it appears.
    ///     Minimum Length: 1
    ///     Maximum Length: 4.
    ///     This allows for a number between 0 and 9999 to be specified.
    ///
    pub type SI = i16;
    ///
    /// 2A.3.72 SNM - string of telephone number digits
    ///
    /// # Definition:
    ///
    ///     A string whose characters are limited to "+" and/or the decimal digits 0 through 9. As
    ///     a string, leading zeros are always considered significant.
    ///     Used only in the XTN data type as of v2.7.
    ///     Minimum Length: 1
    ///     Maximum Length: Not specified for the type. May be specified in the context of use
    ///     SNM is used for telephone numbers, so it is never appropriate to truncate values of type SNM.
    ///
    pub type SNM = V2String;
    pub enum V2TypeIDs{
        V2DT,
        BOOL,
        INTEGER,
        DECIMAL,
        V2STRING,
    }
}

