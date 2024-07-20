use std::cmp::max;
use std::fmt::format;
use unicode_segmentation::UnicodeSegmentation;

/****************************Constants**************************************/
const ESCAPED_STRING_WINDOW: usize = 6;
const ASCII_ESCAPE_CHAR: char = '\\';
const MIN_ASCII_READABLE: char = ' ';
const MAX_ASCII_READABLE: char = '~';
const READABLE_ASCII: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/****************************Traits*****************************************/

///
/// Implemented indexing trait for String and str which uses the UnicodeSegmentation facilities to
/// enable grapheme iteration by default. There could be some performance penalty, but it will allow
/// for native Unicode support to the best extent possible.
///
pub trait UTFStringExtensions {
    fn count_graphemes(&self) -> usize;
    fn get_grapheme(&self, index: usize) -> &str;
    #[inline(always)]
    fn get_grapheme_window(&self, min: usize, max: usize) -> String {
        let mut window: String = String::with_capacity(max - min);
        for i in min..max {
            window += self.get_grapheme(i);
        }
        println!("{}", window);
        window
    }
    #[inline(always)]
    fn get_grapheme_string(&self, end_pattern: &str, offset: usize) -> String {
        let grapheme_count = self.count_graphemes();
        let mut window: String = String::with_capacity(ESCAPED_STRING_WINDOW);
        for i in offset..grapheme_count {
            let g = self.get_grapheme(i);
            if g == end_pattern {
                return window;
            } else {
                window += g;
            }
        }
        window
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

/*****************************String Types***************************************/


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
pub fn unescape_string(escaped_str: &str) -> Result<String, String> {
    let str_size = escaped_str.count_graphemes();
    let mut result: String = String::with_capacity(escaped_str.len());
    let mut i = 0;
    while i < str_size {
        let seq_start = escaped_str.get_grapheme(i);
        match seq_start {
            "\\" => {
                let escape_seq = escaped_str.get_grapheme_string(" ", i);
                let c= match unescape(&escape_seq) {
                    Ok(c) => c,
                    Err(why) => escape_seq.clone()
                };
                result += &c;
                i += &escape_seq.count_graphemes();
            },
            _ => {
                result += seq_start;
                i += 1;
            }
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
pub fn unescape(escaped_str: &str) -> Result<String, String> {
    let lower_case = escaped_str.to_lowercase();
    match &lower_case[0..2] {
        // Hex notation case.
        "\\x" => number_to_grapheme(&hex_to_number(&lower_case[2..])?),
        // Unicode notation case
        "\\u" => number_to_grapheme(&hex_to_number(&lower_case[2..])?),
        // Single byte notation case
        "\\c" => number_to_grapheme(&hex_to_number(&lower_case[2..])?),
        // Unicode notation case
        "\\o" => number_to_grapheme(&octal_to_number(&lower_case[2..])?),
        // Multibyte notation case
        "\\m" => match lower_case.count_graphemes() {
            8 => Ok(number_to_grapheme(&hex_to_number(&lower_case[2..4])?)? +
                &number_to_grapheme(&hex_to_number(&lower_case[4..6])?)? +
                &number_to_grapheme(&hex_to_number(&lower_case[6..])?)?),
            6 => number_to_grapheme(&octal_to_number(&lower_case[2..])?),
            _ => Err(format!("Unknown multibyte sequence. Cannot decode {}", lower_case))
        }
        // Single byte codes.
        _ => Ok(unescape_control(&lower_case)?.to_string())
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
        "\\\\" => Ok(ASCII_ESCAPE_CHAR),
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
fn number_to_grapheme(num: &u32) -> Result<String, String> {
    match char::from_u32(*num) {
        Some(result) => Ok(result.to_string()),
        None => Err(format!("Failed to cast number to character! Number {}", num))
    }
}

///
/// This function will scan through an unescaped string and escape any characters outside the
/// ASCII printable range.
///
pub fn escape_str(in_str: &str) -> String {
    let max_str_size = 4 * in_str.len();
    let mut result = String::with_capacity(max_str_size);
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
pub fn escape(unescaped_str: &str) -> String {
    let escaped_value = unescaped_str.escape_default().to_string();
    escaped_value.replace("{", "").replace("}", "")
}
