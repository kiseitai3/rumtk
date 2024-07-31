use std::cmp::max;
use std::fmt::format;
use std::os::unix::ffi::OsStringExt;
use unicode_segmentation::UnicodeSegmentation;
pub use compact_str::{CompactString, CompactStringExt, ToCompactString, format_compact};
use chardetng::EncodingDetector;

/**************************** Constants**************************************/
const ESCAPED_STRING_WINDOW: usize = 6;
const ASCII_ESCAPE_CHAR: char = '\\';
const MIN_ASCII_READABLE: char = ' ';
const MAX_ASCII_READABLE: char = '~';
const READABLE_ASCII: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

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

    fn get_grapheme(&self, index: usize) -> &str;

    #[inline(always)]
    fn get_grapheme_window(&self, min: usize, max: usize) -> RUMString {
        let mut window: String = String::with_capacity(max - min);
        for i in min..max {
            window += self.get_grapheme(i);
        }
        RUMString::from(window)
    }

    #[inline(always)]
    fn get_grapheme_string(&self, end_pattern: &str, offset: usize) -> RUMString {
        let grapheme_count = self.count_graphemes();
        let mut window: String = String::with_capacity(ESCAPED_STRING_WINDOW);
        for i in offset..grapheme_count {
            let g = self.get_grapheme(i);
            if g == end_pattern {
                return RUMString::from(window);
            } else {
                window += g;
            }
        }
        RUMString::from(window)
    }

    #[inline(always)]
    fn find_grapheme(&self, pattern: &str, offset: usize) -> usize {
        let grapheme_count = self.count_graphemes();
        for i in offset..grapheme_count {
            if self.get_grapheme(i) == pattern {
                return i;
            }
        }
        grapheme_count
    }
}

pub trait RUMStringConversions: ToString {
    fn to_rumstring(&self) -> RUMString {
        RUMString::from(self.to_string())
    }
}

impl UTFStringExtensions for RUMString {
    #[inline(always)]
    fn count_graphemes(&self) -> usize {
        self.graphemes(true).count()
    }

    #[inline(always)]
    fn get_grapheme(&self, index: usize) -> &str {
        self.graphemes(true).nth(index).unwrap()
    }
}

impl RUMStringConversions for RUMString { }

impl UTFStringExtensions for str {
    #[inline(always)]
    fn count_graphemes(&self) -> usize {
        self.graphemes(true).count()
    }

    #[inline(always)]
    fn get_grapheme(&self, index: usize) -> &str {
        self.graphemes(true).nth(index).unwrap()
    }
}

impl RUMStringConversions for str { }

impl RUMStringConversions for char { }

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

/// Take date time string in the format YYYY\[MMDDHHmmss\] and decompose it into numerical
/// date time components.
/// Meaning, we take a string and we return a tuple of numbers.
pub fn decompose_dt_str(dt_str: &RUMString) -> (u16,u8,u8,u8,u8,u8) {
    let mut year: u16 = 0;
    let mut month: u8 = 0;
    let mut day: u8 = 0;
    let mut hour: u8 = 0;
    let mut minute: u8 = 0;
    let mut second: u8 = 0;

    match dt_str.len() {
        4 => {
            year = dt_str.parse::<u16>().unwrap();
        }
        6 => {
            year = dt_str[0..4].parse::<u16>().unwrap();
            month = dt_str[4..].parse::<u8>().unwrap();
        }
        8 => {
            year = dt_str[0..4].parse::<u16>().unwrap();
            month = dt_str[4..6].parse::<u8>().unwrap();
            day = dt_str[6..].parse::<u8>().unwrap();
        }
        10 => {
            year = dt_str[0..4].parse::<u16>().unwrap();
            month = dt_str[4..6].parse::<u8>().unwrap();
            day = dt_str[6..8].parse::<u8>().unwrap();
            hour = dt_str[8..].parse::<u8>().unwrap();
        }
        12 => {
            year = dt_str[0..4].parse::<u16>().unwrap();
            month = dt_str[4..6].parse::<u8>().unwrap();
            day = dt_str[6..8].parse::<u8>().unwrap();
            hour = dt_str[8..10].parse::<u8>().unwrap();
            minute = dt_str[10..].parse::<u8>().unwrap();
        }
        14 => {
            year = dt_str[0..4].parse::<u16>().unwrap();
            month = dt_str[4..6].parse::<u8>().unwrap();
            day = dt_str[6..8].parse::<u8>().unwrap();
            hour = dt_str[8..10].parse::<u8>().unwrap();
            minute = dt_str[10..12].parse::<u8>().unwrap();
            second = dt_str[12..].parse::<u8>().unwrap();
        }
        _ => {

        }
    }
    (year, month, day, hour, minute, second)
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
    match encoding.decode_without_bom_handling_and_without_replacement(&src){
        Some(res) => {
            RUMString::from(res)
        },
        None => RUMString::from_utf8(src).unwrap()
    }
}

///
/// This function will scan through an escaped string and unescape any escaped characters.
/// We collect these characters as a byte vector.
/// Finally, we do a decode pass on the vector to re-encode the bytes **hopefully right** into a
/// valid UTF-8 string.
///
pub fn unescape_string(escaped_str: &str) -> Result<RUMString, RUMString> {
    let str_size = escaped_str.count_graphemes();
    let mut result: Vec<u8> = Vec::with_capacity(escaped_str.len());
    let mut i = 0;
    while i < str_size {
        let seq_start = escaped_str.get_grapheme(i);
        match seq_start {
            "\\" => {
                let escape_seq = escaped_str.get_grapheme_string(" ", i);
                let mut c= match unescape(&escape_seq) {
                    Ok(c) => c,
                    Err(_why) => Vec::from(escape_seq.as_bytes())
                };
                result.append(&mut c);
                i += &escape_seq.count_graphemes();
            },
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
        },
        // Unicode notation case, we need to do an extra step or we will lose key bytes.
        "\\u" => {
            let byte_str = number_to_char_unchecked(&hex_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        },
        // Single byte notation case
        "\\c" => {
            let byte_str = number_to_char_unchecked(&hex_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        },
        // Unicode notation case
        "\\o" => {
            let byte_str = number_to_char_unchecked(&octal_to_number(&lower_case[2..6])?);
            bytes.append(&mut byte_str.as_bytes().to_vec());
        },
        // Multibyte notation case
        "\\m" => match lower_case.count_graphemes() {
            8 => {
                bytes.push(hex_to_byte(&lower_case[2..4])?);
                bytes.push(hex_to_byte(&lower_case[4..6])?);
                bytes.push(hex_to_byte(&lower_case[6..8])?);
            },
            6 => {
                bytes.push(hex_to_byte(&lower_case[2..4])?);
                bytes.push(hex_to_byte(&lower_case[4..6])?);
            },
            _ => return Err(format_compact!("Unknown multibyte sequence. Cannot decode {}", lower_case))
        },
        // Custom encoding
        "\\z" => bytes.append(&mut lower_case.as_bytes().to_vec()),
        // Single byte codes.
        _ => bytes.push(unescape_control_byte(&lower_case)?)
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

        _ => Err(format_compact!("Unknown escape sequence? Sequence: {}!", escaped_str))
    }
}

///
/// Unescape basic character
/// We use pattern matching to map the basic escape character to its corresponding integer value.
///
fn unescape_control_byte(escaped_str: &str) -> Result<u8, RUMString> {
    match escaped_str {
        // Common control sequences
        "\\t" => Ok(9),                       // Tab/Character Tabulation
        "\\b" => Ok(8),                       // Backspace
        "\\n" => Ok(10),                      // New line/ Line Feed character
        "\\r" => Ok(13),                      // Carriage Return character
        "\\f" => Ok(12),                      // Form Feed
        "\\s" => Ok(32),                      // Space
        "\\\\" => Ok(27),                     // Escape
        "\\'" => Ok(39),                      // Single quote
        "\\\"" => Ok(34),                     // Double quote
        "\\0" => Ok(0),                       // Null character
        "\\v" => Ok(11),                      // Vertical Tab/Line Tabulation
        "\\a" => Ok(7),                       // Alert bell
        // Control sequences by hex
        //Err(format_compact!("Unknown escape sequence? Sequence: {}!", escaped_str))
        _ => hex_to_byte(&escaped_str[2..])
    }
}

///
/// Turn hex string to number (u32)
///
fn hex_to_number(hex_str: &str) -> Result<u32, RUMString> {
    match u32::from_str_radix(&hex_str, 16) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!("Failed to parse string with error {}! Input string {} \
        is not hex string!", val, hex_str))
    }
}

///
/// Turn hex string to byte (u8)
///
fn hex_to_byte(hex_str: &str) -> Result<u8, RUMString> {
    match u8::from_str_radix(&hex_str, 16) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!("Failed to parse string with error {}! Input string {} \
        is not hex string!", val, hex_str))
    }
}

///
/// Turn octal string to number (u32)
///
fn octal_to_number(hoctal_str: &str) -> Result<u32, RUMString> {
    match u32::from_str_radix(&hoctal_str, 8) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!("Failed to parse string with error {}! Input string {} \
        is not an octal string!", val, hoctal_str))
    }
}

///
/// Turn octal string to byte (u32)
///
fn octal_to_byte(hoctal_str: &str) -> Result<u8, RUMString> {
    match u8::from_str_radix(&hoctal_str, 8) {
        Ok(result) => Ok(result),
        Err(val) => Err(format_compact!("Failed to parse string with error {}! Input string {} \
        is not an octal string!", val, hoctal_str))
    }
}

///
/// Turn number to UTF-8 char
///
fn number_to_char(num: &u32) -> Result<RUMString, RUMString> {
    match char::from_u32(*num) {
        Some(result) => Ok(result.to_rumstring()),
        None => Err(format_compact!("Failed to cast number to character! Number {}", num))
    }
}

///
/// Turn number to UTF-8 char. Normally, calling from_u32 checks if the value is a valid character.
/// This version uses the less safe from_u32_unchecked() function because we want to get the bytes
/// and deal with validity at a higher layer.
///
fn number_to_char_unchecked(num: &u32) -> RUMString {
    unsafe {
        char::from_u32_unchecked(*num).to_rumstring()
    }
}

///
/// This function will scan through an unescaped string and escape any characters outside the
/// ASCII printable range.
///
pub fn escape_str(in_str: &str) -> RUMString {
    let max_str_size = 4 * in_str.len();
    let mut result = RUMString::with_capacity(max_str_size);
    for c in in_str.chars() {
        if c < MIN_ASCII_READABLE || c > MAX_ASCII_READABLE {
            result += &escape(c.to_string().as_str());
        } else {
            result.push(c);
        }
    }
    result
}

///
/// Turn UTF-8 character into escaped character sequence
///
pub fn escape(unescaped_str: &str) -> RUMString {
    let escaped_value = unescaped_str.escape_default().to_string();
    escaped_value.replace("{", "").replace("}", "").to_rumstring()
}
