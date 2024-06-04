
mod hl7_v2_types {
    use chrono::prelude::*;
    use unicode_segmentation::UnicodeSegmentation;
    use rumtk_core::count_tokens_ignoring_pattern;

    type V2String = String;

    const V2DATETIME_THOUSAND_TICK: u32 = 1000;
    const V2DATETIME_MIRCRO_LENGTH: u8 = 6;
    struct V2DateTime {
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
        fn new(self) -> V2DateTime {
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

        fn from_utc_datetime(self, utc_dt: &DateTime<Utc>) -> V2DateTime {
            V2DateTime{
                year: utc_dt.year() as u16,
                month: utc_dt.month() as u8,
                day: utc_dt.day() as u8,
                hour: utc_dt.hour() as u8,
                minute: utc_dt.minute() as u8,
                second: utc_dt.second() as u8,
                microsecond: utc_dt.nanosecond() / V2DATETIME_THOUSAND_TICK,
                offset: utc_dt.offset().to_string(),
            }
        }

        fn from_v2_string(self, item: V2String) -> V2DateTime {
            let grapheme_vector: Vec<&str> = item.graphemes(true).collect();
            let dt_vec: Vec<&str> = item.split('.').collect();
            let mut ms_vec: Vec<&str> = dt_vec[-1].split('+').collect();
            if count_tokens_ignoring_pattern(&ms_vec, &String::from(" ")) < 2 {
                ms_vec = dt_vec[-1].split('-').collect();
            }



            V2DateTime{ year, month, day, hour, minute, second, microsecond, offset}
        }

        fn as_utc_string(self) -> String {
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

        fn as_utc_datetime(self) -> DateTime<Utc> {
            self.as_utc_string().parse().unwrap()
        }
    }
}