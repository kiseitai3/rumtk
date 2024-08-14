
pub mod rumtk_search {
    use regex::{Regex, Captures};
    use crate::cache::{RUMCache, AHashMap, Lazy};
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
        println!("??????");
        unsafe {
            println!("??????");
            println!("{:?}", re_cache.get(expr));
            match re_cache.get(expr) {
                Some(re) => re,
                None => {
                    re_cache.insert(RUMString::from(expr), compile_regex(expr));
                    println!("Inserted");
                    re_cache.get(expr).unwrap()
                }
            }
        }
    }

    fn compile_regex(expr: &str) -> Regex {
        Regex::new(expr).unwrap()
    }

    pub fn string_search_captures(input: &str, expr: &str, default: &str) -> SearchGroups {
        let re = get_or_set_regex_from_cache(expr);
        //let re = compile_regex(expr);
        let names: Vec<&str> = re.capture_names().skip(1).map(|x| x.unwrap_or_else(|| "")).collect();
        let mut clean_names: Vec<&str> = Vec::with_capacity(names.len());
        let mut groups = SearchGroups::default();

        for name in &names {
            if name.len() > 0 {
                clean_names.push(name);
            }
        }

        if clean_names.len() == 0 {
            return groups;
        }

        for name in &clean_names {
            groups.insert(RUMString::from(name.to_string()), RUMString::from(default));
        }

        for cap in re.captures_iter(input).map(|c| c) {
            for name in &clean_names {
                let val = cap.name(name).map_or("", |s| s.as_str());
                if val.len() > 0 {
                    groups.insert(RUMString::from(name.to_string()), RUMString::from(val));
                }
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