/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */


pub mod rumtk_search {
    use regex::{Regex};
    use crate::cache::{LazyRUMCache, AHashMap, new_cache, get_or_set_from_cache};
    use crate::strings::{RUMString, CompactStringExt};
    /**************************** Globals **************************************/
    static mut re_cache: RegexCache = new_cache();
    /**************************** Constants**************************************/
    const DEFAULT_REGEX_CACHE_PAGE_SIZE: usize = 10;
    /**************************** Types *****************************************/
    pub type RegexCache = LazyRUMCache<RUMString, Regex>;
    pub type SearchGroups = AHashMap<RUMString, RUMString>;
    pub type CapturedList = Vec<RUMString>;

    /**************************** Traits ****************************************/

    /**************************** Helpers ***************************************/
    fn compile_regex(expr: &RUMString) -> Regex {
        Regex::new(expr).unwrap()
    }

    ///
    /// Finds all of the named regex captures and generates a hash table with the results assorted
    /// into key-value pairs. The keys are the names found in the regex expression. The value is
    /// the match corresponding to the named capture.
    ///
    /// This function returns an instance of SearchGroup which is the hash map.
    ///
    pub fn string_search_named_captures(input: &str, expr: &str, default: &str) -> SearchGroups {
        let re = unsafe {
            get_or_set_from_cache(&mut re_cache, &RUMString::from(expr), compile_regex)
        };
        let names: Vec<&str> = re.capture_names().skip(1).map(|x| x.unwrap_or_else(|| "")).collect();
        let mut clean_names: Vec<&str> = Vec::with_capacity(names.len());
        let mut groups = SearchGroups::with_capacity(DEFAULT_REGEX_CACHE_PAGE_SIZE);

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

    ///
    /// Finds all of the regex captures regardless of name status and compile them into a list
    /// of strings. Elsewhere, this provides a simple way to iterate through the contents that
    /// were inside a group \(\).
    ///
    /// This function returns an instance of CapturedList which is the list of strings.
    ///
    pub fn string_search_all_captures(input: &str, expr: &str, default: &str) -> CapturedList {
        let re = unsafe {
            get_or_set_from_cache(&mut re_cache, &RUMString::from(expr), compile_regex)
        };
        let mut capture_list = CapturedList::with_capacity(DEFAULT_REGEX_CACHE_PAGE_SIZE);

        for caps in re.captures_iter(input) {
            for c in caps.iter().skip(1) {
                let c_str = c.unwrap().as_str();
                capture_list.push(RUMString::from(c_str));
            }
        }

        capture_list
    }

    ///
    /// Given a string input and a compiled RegEx, look for all matches and put them in a string
    /// list for easy iteration/access.
    ///
    pub fn string_list(input: &str, re: &Regex) -> CapturedList {
        let mut list: Vec<RUMString> = Vec::with_capacity(DEFAULT_REGEX_CACHE_PAGE_SIZE);
        for itm in re.find_iter(input) {
            list.push(RUMString::from(itm.as_str()));
        }
        list
    }


    ///
    /// Given a string input and a RegEx string,
    ///
    ///     - Compile the regex if not done so already.
    ///     - Do a string search for all regex matches.
    ///     - Collapse/join the matches into a single output string using join_pattern as the join fragment.
    ///
    /// Use \" \" in join_pattern if you wish to have spaces in between matches.
    ///
    pub fn string_search(input: &str, expr: &str, join_pattern: &str) -> RUMString {
        let re = unsafe {
            get_or_set_from_cache(&mut re_cache, &RUMString::from(expr), compile_regex)
        };
        string_list(input, &re).join_compact(join_pattern)
    }
}
