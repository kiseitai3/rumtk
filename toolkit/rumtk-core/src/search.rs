
pub mod rumtk_search {
    use regex::{Regex, Captures};
    use once_cell::unsync::Lazy;
    use crate::cache::{RUMCache, AHashMap};
    use crate::strings::{RUMString, format_compact, UTFStringExtensions, RUMStringConversions, CompactStringExt};
    /**************************** Globals **************************************/
    static mut re_cache: Lazy<RegexCache> = Lazy::new(|| {
        RegexCache::default()
    });

    /**************************** Constants**************************************/

    /**************************** Types *****************************************/
    pub type RegexCache = RUMCache<RUMString, Regex>;
    pub type SearchGroups = AHashMap<RUMString, RUMString>;
    pub type CapturedList = Vec<RUMString>;

    /**************************** Traits ****************************************/

    /**************************** Helpers ***************************************/

    fn get_or_set_regex_from_cache(expr: &str) -> &Regex {
        unsafe {
            match re_cache.contains_key(expr) {
                true => re_cache.get(expr).unwrap(),
                false => {
                    re_cache.insert(RUMString::from(expr), Regex::new(expr).unwrap());
                    re_cache.get(expr).unwrap()
                }
            }
        }
    }

    fn get_capture_list<'b>(input: &'b str, re: &'b Regex) -> Vec<Captures<'b>> {
        let mut captures: Vec<Captures> = Vec::default();
        for cap in re.captures_iter(input).map(|c| c) {
            captures.push(cap);
        }
        captures
    }

    pub fn string_search_captures(input: &str, expr: &str) -> SearchGroups {
        let re = get_or_set_regex_from_cache(expr);
        let names: Vec<&str> = re.capture_names().skip(1).map(|x| x.unwrap()).collect();
        let mut groups = SearchGroups::default();

        if names.len() == 0 {
            return groups;
        }

        for name in &names {
            groups.insert(RUMString::from(name.to_string()), RUMString::default());
        }

        for cap in get_capture_list(input, re) {
            for name in &names {
                groups.insert(RUMString::from(name.to_string()), RUMString::from(cap.name(name).map_or("", |s| s.as_str())));
            }
        }

        groups
    }

    pub fn string_list(input: &str, re: &Regex) -> CapturedList {
        let mut list: Vec<RUMString> = Vec::default();
        for itm in re.find_iter(input) {
            list.push(RUMString::from(itm.as_str()));
        }
        list
    }

    pub fn string_search(input: &str, expr: &str, join_pattern: &str) -> RUMString {
        let re = get_or_set_regex_from_cache(expr);
        string_list(input, &re).join_compact(join_pattern)
    }
}
