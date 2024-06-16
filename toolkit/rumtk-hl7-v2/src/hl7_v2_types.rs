
pub mod v2_types {
    use chrono::prelude::*;
    use rumtk_core::strings::{count_tokens_ignoring_pattern, decompose_dt_str};
    use rumtk_core::maths::generate_tenth_factor;
    use crate::hl7_v2_constants::{V2_DATETIME_MIRCRO_LENGTH, V2_DATETIME_THOUSAND_TICK};

    pub type V2String = String;
    pub struct V2DateTime {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        offset: String
    }

    impl V2DateTime {
        pub fn new() -> V2DateTime {
            V2DateTime{
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
                microsecond: 0,
                offset: String::from("0"),
            }
        }

        pub fn from_utc_datetime(utc_dt: &DateTime<Utc>) -> V2DateTime {
            V2DateTime{
                year: utc_dt.year() as u16,
                month: utc_dt.month() as u8,
                day: utc_dt.day() as u8,
                hour: utc_dt.hour() as u8,
                minute: utc_dt.minute() as u8,
                second: utc_dt.second() as u8,
                microsecond: utc_dt.nanosecond() / (V2_DATETIME_THOUSAND_TICK as u32),
                offset: utc_dt.offset().to_string(),
            }
        }

        pub fn from_v2_string(item: &V2String) -> V2DateTime {
            // Begin decomposing string into discrete components per HL7 DateTime format specs.
            // See https://hl7-definition.caristix.com/v2/HL7v2.8/DataTypes/DTM
            let dt_vec: Vec<&str> = item.split('.').collect();
            let mut ms_vec: Vec<&str> = dt_vec.last().unwrap().split('+').collect();
            if count_tokens_ignoring_pattern(&ms_vec, &String::from(" ")) < 2 {
                ms_vec = dt_vec.last().unwrap().split('-').collect();
            }

            let (year, month, day, hour, minute, second) =
                decompose_dt_str(&String::from(dt_vec[0]));

            // Now let's grab the two components of the vector and generate the microsecond and offset bits.
            let ms_string = ms_vec[0];
            let ms_string_len = ms_string.len();
            let microsecond = match ms_string_len {
                0 => 0,
                _ => ms_string.parse::<u32>().unwrap() *
                    generate_tenth_factor(
                        (V2_DATETIME_MIRCRO_LENGTH - (ms_string_len as u8)) as u32)
            };

            let offset: String = String::from(ms_vec[1]);


            V2DateTime{ year, month, day, hour, minute, second, microsecond, offset}
        }

        fn as_utc_string(&self) -> String {
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

        fn as_utc_datetime(&self) -> DateTime<Utc> {
            self.as_utc_string().parse().unwrap()
        }
    }
}