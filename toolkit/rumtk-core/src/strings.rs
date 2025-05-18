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
use crate::core::{is_unique, RUMResult};
use chardetng::EncodingDetector;
pub use compact_str::{format_compact, CompactString, CompactStringExt, ToCompactString};
use encoding_rs::Encoding;
use std::fmt::Display;
use unicode_segmentation::UnicodeSegmentation;
/**************************** Constants**************************************/
const ESCAPED_STRING_WINDOW: usize = 6;
const ASCII_ESCAPE_CHAR: char = '\\';
const MIN_ASCII_READABLE: char = ' ';
const MAX_ASCII_READABLE: char = '~';
pub const EMPTY_STRING: &str = "";
pub const DOT_STR: &str = ".";
pub const EMPTY_STRING_OPTION: Option<&str> = Some("");
pub const READABLE_ASCII: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/**************************** Types *****************************************/
pub type RUMString = CompactString;

/**************************** Traits ****************************************/

///
/// Implemented indexing trait for String and str which uses the UnicodeSegmentation facilities to
/// enable grapheme iteration by default. There could be some performance penalty, but it will allow
/// for native Unicode support to the best extent possible.
///
/// We also enable decoding from Encoding Standard encodings to UTF-8.
///
pub trait UTFStringExtensions {
    fn count_graphemes(&self) -> usize;

    ///
    /// Return a grapheme unit which could span multiple Unicode codepoints or "characters".
    ///
    /// # Note
    /// ```text
    ///     If the grapheme requested does not exists, this method will return a blank string.
    /// ```
    ///
    /// Instead of just retrieving a codepoint as character, I decided to take it a step further and
    /// have support for grapheme selection such that characters in written language like sanskrit
    /// can be properly selected and evaluated.
    ///
    /// [!CAUTION]
    /// This can be an extremely slow operation over large strings since each call to this method
    /// will need to rescan the input string every time we need to look up a grapheme. Unfortunately,
    /// this is a side effect of convenience. To improve performance, call .get_graphemes() once and
    /// then call take_grapheme() over that iterator.
    ///
    fn get_grapheme(&self, index: usize) -> &str;

    fn get_graphemes(&self) -> Vec<&str>;

    fn get_grapheme_chunk(&self, offset: usize) -> Vec<&str>;

    #[inline(always)]
    fn take_grapheme<'a>(&self, graphemes: &Vec<&'a str>, index: usize) -> RUMString {
        if index >= graphemes.len() {
            return RUMString::from(EMPTY_STRING);
        }
        RUMString::from(graphemes[index])
    }

    #[inline(always)]
    fn get_grapheme_window(&self, min: usize, max: usize, offset: usize) -> RUMString {
        let mut window: RUMString = RUMString::with_capacity(max - min);
        let start = min + offset;
        let end = max + offset;
        let graphemes = self.get_graphemes();
        for i in start..end {
            window += &self.take_grapheme(&graphemes, i);
        }
        window
    }

    #[inline(always)]
    fn get_grapheme_string(&self, end_pattern: &str, offset: usize) -> RUMString {
        let mut window: RUMString = RUMString::with_capacity(ESCAPED_STRING_WINDOW);
        for grapheme in self.get_grapheme_chunk(offset) {
            if grapheme == end_pattern {
                return RUMString::from(window);
            } else {
                window += grapheme;
            }
        }
        RUMString::from(window)
    }

    #[inline(always)]
    fn find_grapheme(&self, pattern: &str, offset: usize) -> &str {
        for grapheme in self.get_grapheme_chunk(offset) {
            if grapheme == pattern {
                return grapheme;
            }
        }
        EMPTY_STRING
    }

    #[inline(always)]
    fn truncate(&self, max_size: usize) -> RUMString {
        self.get_grapheme_window(0, max_size, 0)
    }
}

pub trait AsStr {
    fn as_str(&self) -> &str;
}

pub trait RUMStringConversions: ToString {
    fn to_rumstring(&self) -> RUMString {
        RUMString::from(self.to_string())
    }

    fn to_raw(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

pub trait StringUtils: AsStr + UTFStringExtensions {
    #[inline(always)]
    fn duplicate(&self, count: usize) -> RUMString {
        let mut duplicated = RUMString::with_capacity(count);
        for i in 0..count {
            duplicated += &self.as_str();
        }
        duplicated
    }

    fn is_unique(&self) -> bool {
        let graphemes = self.get_graphemes();
        is_unique(&graphemes)
    }
}

impl UTFStringExtensions for RUMString {
    #[inline(always)]
    fn count_graphemes(&self) -> usize {
        self.graphemes(true).count()
    }

    #[inline(always)]
    fn get_grapheme(&self, index: usize) -> &str {
        self.graphemes(true)
            .nth(index)
            .or(EMPTY_STRING_OPTION)
            .unwrap()
    }

    #[inline(always)]
    fn get_graphemes(&self) -> Vec<&str> {
        self.graphemes(true).collect::<Vec<&str>>()
    }

    #[inline(always)]
    fn get_grapheme_chunk(&self, offset: usize) -> Vec<&str> {
        self.graphemes(true).skip(offset).collect::<Vec<&str>>()
    }
}

impl RUMStringConversions for RUMString {}
impl AsStr for RUMString {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}
impl StringUtils for RUMString {}

impl UTFStringExtensions for str {
    #[inline(always)]
    fn count_graphemes(&self) -> usize {
        self.graphemes(true).count()
    }

    #[inline(always)]
    fn get_grapheme(&self, index: usize) -> &str {
        self.graphemes(true)
            .nth(index)
            .or(EMPTY_STRING_OPTION)
            .unwrap()
    }

    #[inline(always)]
    fn get_graphemes(&self) -> Vec<&str> {
        self.graphemes(true).collect::<Vec<&str>>()
    }

    #[inline(always)]
    fn get_grapheme_chunk(&self, offset: usize) -> Vec<&str> {
        self.graphemes(true).skip(offset).collect::<Vec<&str>>()
    }
}

impl RUMStringConversions for str {}

impl AsStr for str {
    fn as_str(&self) -> &str {
        self
    }
}

impl StringUtils for str {}

impl RUMStringConversions for char {}

pub trait RUMArrayConversions {
    fn to_rumstring(&self) -> RUMString;
}

impl RUMArrayConversions for Vec<u8> {
    fn to_rumstring(&self) -> RUMString {
        self.as_slice().to_rumstring()
    }
}

impl RUMArrayConversions for &[u8] {
    fn to_rumstring(&self) -> RUMString {
        RUMString::from_utf8(&self).unwrap()
    }
}

/**************************** Helpers ***************************************/

pub fn count_tokens_ignoring_pattern(vector: &Vec<&str>, string_token: &RUMString) -> usize {
    let mut count: usize = 0;
    for tok in vector.iter() {
        if string_token != tok {
            count += 1;
        }
    }
    count
}

///
/// Implements decoding this string from its auto-detected encoding to UTF-8.
/// Failing that we assume the string was encoded in UTF-8 and return a copy.
///
/// Note => Decoding is facilitated via the crates chardet-ng and encoding_rs.
///
pub fn try_decode(src: &[u8]) -> RUMString {
    let mut detector = EncodingDetector::new();
    detector.feed(&src, true);
    let encoding = detector.guess(None, true);
    decode(src, encoding)
}

///
/// Implements decoding this string from a specific encoding to UTF-8.
///
/// Note => Decoding is facilitated via the crates chardet-ng and encoding_rs.
///
pub fn try_decode_with(src: &[u8], encoding_name: &str) -> RUMString {
    let encoding = match Encoding::for_label(encoding_name.as_bytes()) {
        Some(v) => v,
        None => return RUMString::from(""),
    };
    decode(src, encoding)
}

///
/// Implements decoding of input with encoder.
///
/// Note => Decoding is facilitated via the crate encoding_rs.
///
fn decode(src: &[u8], encoding: &'static Encoding) -> RUMString {
    match encoding.decode_without_bom_handling_and_without_replacement(&src) {
        Some(res) => RUMString::from(res),
        None => RUMString::from_utf8(src).unwrap(),
    }
}

///
/// This function will scan through an escaped string and unescape any escaped characters.
/// We collect these characters as a byte vector.
/// Finally, we do a decode pass on the vector to re-encode the bytes **hopefully right** into a
/// valid UTF-8 string.
///
/// This function focuses on reverting the result of [escape], whose output is meant for HL7.
///
pub fn unescape_string(escaped_str: &str) -> RUMResult<RUMString> {
    let str_size = escaped_str.count_graphemes();
    let mut result: Vec<u8> = Vec::with_capacity(escaped_str.len());
    let mut i = 0;
    while i < str_size {
        let seq_start = escaped_str.get_grapheme(i);
        match seq_start {
            "\\" => {
                let escape_seq = escaped_str.get_grapheme_string(" ", i);
                let mut c = match unescape(&escape_seq) {
                    Ok(c) => c,
                    Err(_why) => Vec::from(escape_seq.as_bytes()),
                };
                result.append(&mut c);
                i += &escape_seq.count_graphemes();
            }
            _ => {
                result.append(&mut Vec::from(seq_start.as_bytes()));
                i += 1;
            }
        }
    }
    Ok(try_decode(result.as_slice()))
}

///
/// Turn escaped character sequence into the equivalent UTF-8 character
/// This function accepts \o, \x and \u formats.
/// This function will also attempt to unescape the common C style control characters.
/// Anything else needs to be expressed as hex or octal patterns with the formats above.
///
/// If I did this right, I should get the "raw" byte sequence out of the escaped string.
/// We can then use the bytes and attempt a decode() to figure out the string encoding and
/// get the correct conversion to UTF-8. **Fingers crossed**
///
pub fn unescape(escaped_str: &str) -> Result<Vec<u8>, RUMString> {
    let lower_case = escaped_str.to_lowercase();
    let mut bytes: Vec<u8> = Vec::with_capacity(3);
    match &lower_case[0..2] {
        // Hex notation case. Assume we are getting xxyy bytes
        "\\x" => {
            let byte_str = number_to_char_unchecked(&hex_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        }
        // Unicode notation case, we need to do an extra step or we will lose key bytes.
        "\\u" => {
            let byte_str = number_to_char_unchecked(&hex_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        }
        // Single byte notation case
        "\\c" => {
            let byte_str = number_to_char_unchecked(&hex_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        }
        // Unicode notation case
        "\\o" => {
            let byte_str = number_to_char_unchecked(&octal_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        }
        // Multibyte notation case
        "\\m" => match lower_case.count_graphemes() {
            8 => {
                bytes.push(hex_to_byte(&lower_case[2..4])?);
                bytes.push(hex_to_byte(&lower_case[4..6])?);
                bytes.push(hex_to_byte(&lower_case[6..8])?);
            }
            6 => {
                bytes.push(hex_to_byte(&lower_case[2..4])?);
                bytes.push(hex_to_byte(&lower_case[4..6])?);
            }
            _ => {
                return Err(format_compact!(
                    "Unknown multibyte sequence. Cannot decode {}",
                    lower_case
                ))
            }
        },
        // Custom encoding
        "\\z" => bytes.append(&mut lower_case.as_bytes().to_vec()),
        // Single byte codes.
        _ => bytes.push(unescape_control_byte(&lower_case)?),
    }
    Ok(bytes)
}

///
/// Unescape basic character
/// We use pattern matching to map the basic escape character to its corresponding integer value.
///
fn unescape_control(escaped_str: &str) -> Result<char, RUMString> {
    match escaped_str {
        // Common control sequences
        "\\t" => Ok('\t'),
        "\\b" => Ok('\x08'),
        "\\n" => Ok('\n'),
        "\\r" => Ok('\r'),
        "\\f" => Ok('\x14'),
        "\\s" => Ok('\x20'),
        "\\\\" => Ok(ASCII_ESCAPE_CHAR),
        "\\'" => Ok('\''),
        "\\\"" => Ok('\"'),
        "\\0" => Ok('\0'),
        "\\v" => Ok('\x0B'),
        "\\a" => Ok('\x07'),
        // Control sequences by
        _ => Err(format_compact!(
            "Unknown escape sequence? Sequence: {}!",
            escaped_str
        )),
    }
}

///
/// Unescape basic character
/// We use pattern matching to map the basic escape character to its corresponding integer value.
///
fn unescape_control_byte(escaped_str: &str) -> Result<u8, RUMString> {
    match escaped_str {
        // Common control sequences
        "\\t" => Ok(9),   // Tab/Character Tabulation
        "\\b" => Ok(8),   // Backspace
        "\\n" => Ok(10),  // New line/ Line Feed character
        "\\r" => Ok(13),  // Carriage Return character
        "\\f" => Ok(12),  // Form Feed
        "\\s" => Ok(32),  // Space
        "\\\\" => Ok(27), // Escape
        "\\'" => Ok(39),  // Single quote
        "\\\"" => Ok(34), // Double quote
        "\\0" => Ok(0),   // Null character
        "\\v" => Ok(11),  // Vertical Tab/Line Tabulation
        "\\a" => Ok(7),   // Alert bell
        // Control sequences by hex
        //Err(format_compact!("Unknown escape sequence? Sequence: {}!", escaped_str))
        _ => hex_to_byte(&escaped_str[2..]),
    }
}

///
/// Turn hex string to number (u32)
///
fn hex_to_number(hex_str: &str) -> Result<u32, RUMString> {
    match u32::from_str_radix(&hex_str, 16) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!(
            "Failed to parse string with error {}! Input string {} \
        is not hex string!",
            val,
            hex_str
        )),
    }
}

///
/// Turn hex string to byte (u8)
///
fn hex_to_byte(hex_str: &str) -> Result<u8, RUMString> {
    match u8::from_str_radix(&hex_str, 16) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!(
            "Failed to parse string with error {}! Input string {} \
        is not hex string!",
            val,
            hex_str
        )),
    }
}

///
/// Turn octal string to number (u32)
///
fn octal_to_number(hoctal_str: &str) -> Result<u32, RUMString> {
    match u32::from_str_radix(&hoctal_str, 8) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!(
            "Failed to parse string with error {}! Input string {} \
        is not an octal string!",
            val,
            hoctal_str
        )),
    }
}

///
/// Turn octal string to byte (u32)
///
fn octal_to_byte(hoctal_str: &str) -> Result<u8, RUMString> {
    match u8::from_str_radix(&hoctal_str, 8) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!(
            "Failed to parse string with error {}! Input string {} \
        is not an octal string!",
            val,
            hoctal_str
        )),
    }
}

///
/// Turn number to UTF-8 char
///
fn number_to_char(num: &u32) -> Result<RUMString, RUMString> {
    match char::from_u32(*num) {
        Some(result) => Ok(result.to_rumstring()),
        None => Err(format_compact!(
            "Failed to cast number to character! Number {}",
            num
        )),
    }
}

///
/// Turn number to UTF-8 char. Normally, calling from_u32 checks if the value is a valid character.
/// This version uses the less safe from_u32_unchecked() function because we want to get the bytes
/// and deal with validity at a higher layer.
///
fn number_to_char_unchecked(num: &u32) -> RUMString {
    unsafe { char::from_u32_unchecked(*num).to_rumstring() }
}

///
/// Turn UTF-8 character into escaped character sequence as expected in HL7
///
/// # Example
/// ```
///  use rumtk_core::strings::{escape};
///  let message = "I ❤ my wife!";
///  let escaped_message = escape(&message);
///  assert_eq!("I \\u2764 my wife!", &escaped_message, "Did not get expected escaped string! Got {}!", &escaped_message);
///```
///
pub fn escape(unescaped_str: &str) -> RUMString {
    basic_escape(unescaped_str)
        .replace("{", "")
        .replace("}", "")
        .to_rumstring()
}

///
/// Escape UTF-8 characters in UTF-8 string that are beyond ascii range
///
/// # Example
/// ```
///  use rumtk_core::strings::basic_escape;
///  let message = "I ❤ my wife!";
///  let escaped_message = basic_escape(&message);
///  assert_eq!("I \\u{2764} my wife!", &escaped_message, "Did not get expected escaped string! Got {}!", &escaped_message);
///```
pub fn basic_escape(unescaped_str: &str) -> RUMString {
    unescaped_str.escape_default().to_compact_string()
}

///
/// Removes all non ASCII and all non printable characters from string.
///
pub fn filter_ascii(unescaped_str: &str, closure: fn(char) -> bool) -> RUMString {
    let mut filtered = unescaped_str.to_rumstring();
    filtered.retain(closure);
    filtered
}

///
/// Removes all non ASCII and all non printable characters from string.
///
pub fn filter_non_printable_ascii(unescaped_str: &str) -> RUMString {
    filter_ascii(unescaped_str, |c: char| {
        !c.is_ascii() && (' ' <= c || c <= '~')
    })
}
