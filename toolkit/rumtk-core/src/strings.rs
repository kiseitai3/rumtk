use std::fmt::format;
use unicode_segmentation::UnicodeSegmentation;

/****************************Constants**************************************/
const ESCAPED_STRING_WINDOW: usize = 6;

/****************************Traits*****************************************/

///
/// Implemented indexing trait for String and str which uses the UnicodeSegmentation facilities to
/// enable grapheme iteration by default. There could be some performance penalty, but it will allow
/// for native Unicode support to the best extent possible.
///
pub trait UTFStringExtensions {
    fn count_graphemes(&self) -> usize;
    fn get_grapheme(&self, index: usize) -> &str;
}

impl UTFStringExtensions for String {
    #[inline(always)]
    fn count_graphemes(&self) -> usize {
        self.graphemes(true).count()
    }

    #[inline(always)]
    fn get_grapheme(&self, index: usize) -> &str {
        self.graphemes(true).nth(index).unwrap()
    }
}

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

/*****************************Other string helpers***************************************/

pub fn count_tokens_ignoring_pattern(vector: &Vec<&str>, string_token: &String) -> usize {
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
pub fn decompose_dt_str(dt_str: &String) -> (u16,u8,u8,u8,u8,u8) {
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
/// This function will scan through an escaped string and unescape any escaped characters
///
pub fn unescape_str(escaped_string: &str) -> Result<String, String> {
    let str_size = escaped_string.len();
    let mut result: String = String::with_capacity(str_size);
    for mut i in 0..str_size {
        let remainder = str_size % ESCAPED_STRING_WINDOW;
        let offset = match remainder {
            0 => ESCAPED_STRING_WINDOW,
            _ => remainder
        };
        let window: &str = &escaped_string[i..i + offset];
        if window.len() == ESCAPED_STRING_WINDOW &&
            (window.contains("\\x") || window.contains("\\X") ||
                window.contains("\\u") || window.contains("\\U")) {
            result.push(unescape(&window)?);
            i += ESCAPED_STRING_WINDOW;
        } else if window.contains('\\') {
            result.push(unescape(&window[0..2])?);
            i += 2;
        } else {
            result.push(window.chars().nth(0).unwrap());
        }
    }
    Ok(result)
}

///
/// Turn escaped character sequence into the equivalent UTF-8 character
/// This function accepts \o, \x and \u formats.
/// This function will also attempt to unescape the common C style control characters.
/// Anything else needs to be expressed as hex or octal patterns with the formats above.
///
pub fn unescape(escaped_str: &str) -> Result<char, String> {
    let lower_case = escaped_str.to_lowercase();
    match &lower_case[0..2] {
        // Hex notation case.
        "\\x" => match hex_to_number(&lower_case[2..]) {
            Ok(val) => Ok(number_to_char(&val)?),
            Err(why) => Err(why)
        },
        // Unicode notation case
        "\\u" => match hex_to_number(&lower_case[2..]) {
            Ok(val) => Ok(number_to_char(&val)?),
            Err(why) => Err(why)
        },
        // Unicode notation case
        "\\o" => match octal_to_number(&lower_case[2..]) {
            Ok(val) => Ok(number_to_char(&val)?),
            Err(why) => Err(why)
        },
        // Single byte codes.
        _ => Ok(unescape_control(&lower_case)?)
    }
}

///
/// Unescape basic character
/// We use pattern matching to map the basic escape character to its corresponding integer value.
///
fn unescape_control(escaped_str: &str) -> Result<char, String> {
    match escaped_str {
        // Common control sequences
        "\\t" => Ok('\t'),
        "\\b" => Ok('\x08'),
        "\\n" => Ok('\n'),
        "\\r" => Ok('\r'),
        "\\f" => Ok('\x14'),
        "\\s" => Ok('\x20'),
        "\\\\" => Ok('\\'),
        "\\'" => Ok('\''),
        "\\\"" => Ok('\"'),
        "\\0" => Ok('\0'),
        "\\v" => Ok('\x0B'),
        "\\a" => Ok('\x07'),
        // Control sequences by

        _ => Err(format!("Unknown escape sequence? Sequence: {}!", escaped_str))
    }
}

///
/// Turn hex string to number (u32)
///
fn hex_to_number(hex_str: &str) -> Result<u32, String> {
    match u32::from_str_radix(&hex_str, 16) {
        Ok(result) => Ok(result),
        Err(val) => Err(format!("Failed to parse string with error {}! Input string {} \
        is not hex string!", val, hex_str))
    }
}

///
/// Turn hex string to number (u32)
///
fn octal_to_number(hoctal_str: &str) -> Result<u32, String> {
    match u32::from_str_radix(&hoctal_str, 8) {
        Ok(result) => Ok(result),
        Err(val) => Err(format!("Failed to parse string with error {}! Input string {} \
        is not an octal string!", val, hoctal_str))
    }
}

///
/// Turn number to UTF-8 char
///
fn number_to_char(num: &u32) -> Result<char, String> {
    match char::from_u32(*num) {
        Some(result) => Ok(result),
        None => Err(format!("Failed to cast number to character! Number {}", num))
    }
}

///
/// Turn UTF-8 character into escaped character sequence
///
pub fn escape(unescaped_str: &str) -> String {
    let escaped_value = unescaped_str.escape_default().to_string();
    escaped_value.replace("{", "").replace("}", "")
}
