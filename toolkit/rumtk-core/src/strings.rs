use unicode_segmentation::UnicodeSegmentation;


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

// Other string helpers.

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
/// Turn escaped character sequence into the equivalent UTF-8 character
///
fn unescape(escaped_str: &str) -> Result<char, String> {
    let lower_case = escaped_str.to_lowercase();
    match &lower_case[0..2] {
        "\\x" => match hex_to_number(&lower_case[2..]) {
            Ok(val) => Ok(number_to_char(&val)?),
            Err(why) => Err(why)
        },
        "\\u" => match hex_to_number(&lower_case[2..]) {
            Ok(val) => Ok(number_to_char(&val)?),
            Err(why) => Err(why)
        },
        _ => Ok(unescape_char(&lower_case[1..2].bytes()[0])?)
    }
}

///
/// Unescape basic character
/// We use a lookup table to map the basic escape character to its corresponding integer value.
///
fn unescape_char(escaped_char: &char) -> Result<char, String> {

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
/// Turn number to UTF-8 char
///
fn number_to_char(num: &u32) -> Result<char, String> {
    match char::from_u32(*num) {
        Ok(result) => Ok(result),
        Err(val) => Err(format!("Failed to cast number to character! Error: {}! Number {}",
                                val, num))
    }
}

///
/// Turn UTF-8 character into escaped character sequence
///
fn escape(unescaped_char: &char) -> String {
    unescaped_char.escape_default().to_string()
}