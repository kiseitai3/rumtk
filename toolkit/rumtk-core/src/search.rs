
pub mod rumtk_search {
    use regex::{Regex, Captures};
    use crate::cache::{RUMCache, AHashMap};
    use crate::strings::{RUMString, format_compact, UTFStringExtensions, RUMStringConversions, CompactStringExt};
    /**************************** Globals **************************************/
    static mut re_cache: RUMCache<&str, Regex> = RUMCache::with_capacity(5);

    /**************************** Constants**************************************/

    /**************************** Types *****************************************/
    pub type SearchGroups = AHashMap<str, RUMString>;
    pub type CapturedList = Vec<RUMString>;

    /**************************** Traits ****************************************/

    /**************************** Helpers ***************************************/

    pub fn get_regex_from_cache(expr: &str) -> &Regex {
        unsafe {
            match re_cache.contains_key(expr) {
                true => re_cache.get(expr).unwrap(),
                false => {
                    re_cache.insert(expr, Regex::new(expr).unwrap());
                    re_cache.get(expr).unwrap()
                }
            }
        }
    }

    pub fn string_search_captures(input: &str, expr: &str) -> SearchGroups {
        let re = get_regex_from_cache(expr);
        let names = re.capture_names();

        if !names.len() {
            return SearchGroups::default();
        }

        match re.captures(input) {
            Some(cap) => {
                let mut list = SearchGroups::with_capacity(cap.len());
                for name in names {
                    let name_str = name.unwrap();
                    list.insert(name_str, RUMString::from(cap.name(name_str)));
                }
                list
            },
            None => SearchGroups::default()
        }
    }

    pub fn string_capture_list(input: &str, re: &Regex) -> Vec<RUMString> {
        match re.captures(input) {
            Some(cap) => {
                let mut list: Vec<RUMString> = Vec::with_capacity(cap.len());
                for (a, c) in cap.extract() {
                    list.push(RUMString::from(a));
                }
                list
            },
            None => Vec::default()
        }
    }

    pub fn string_search(input: &str, expr: &str) -> RUMString {
        let re = get_regex_from_cache(expr);
        string_capture_list(input, &re).join_compact(" ")
    }
}
