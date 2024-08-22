
pub mod rumtk_search {
    use regex::{Regex};
    use crate::cache::{LazyRUMCache, AHashMap, new_cache, get_or_set_from_cache};
    use crate::strings::{RUMString, CompactStringExt};
    /**************************** Globals **************************************/
    static mut re_cache: RegexCache = new_cache();
    /**************************** Constants**************************************/
   // const compile_regex: dyn Fn() = || {

    //};
    /**************************** Types *****************************************/
    pub type RegexCache = LazyRUMCache<RUMString, Regex>;
    pub type SearchGroups = AHashMap<RUMString, RUMString>;
    pub type CapturedList = Vec<RUMString>;

    /**************************** Traits ****************************************/

    /**************************** Helpers ***************************************/
/*
    fn get_or_set_regex_from_cache(expr: &str) -> Regex {
        println!("??????");
        unsafe {
            println!("??????");
            println!("{:?}", re_cache.get(expr));
            if re_cache.contains_key(expr) {
                return re_cache.get(expr).unwrap().clone();
            } else {
                re_cache.insert(RUMString::from(expr), compile_regex(expr));
                re_cache.get(expr).unwrap().clone()
            }
        }
    }

    fn compile_regex(expr: &str) -> Regex {
        Regex::new(expr).unwrap()
    }
*/

    fn compile_regex(expr: &RUMString) -> Regex {
        Regex::new(expr).unwrap()
    }

    pub fn string_search_captures(input: &str, expr: &str, default: &str) -> SearchGroups {
        let re = unsafe {
            get_or_set_from_cache(&mut re_cache, &RUMString::from(expr), compile_regex)
        };
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

        let mut c_count = 0;
        for cap in re.captures_iter(input).map(|c| c) {
            c_count += 1;
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
        let re = unsafe {
            get_or_set_from_cache(&mut re_cache, &RUMString::from(expr), compile_regex)
        };
        string_list(input, &re).join_compact(join_pattern)
    }
}
