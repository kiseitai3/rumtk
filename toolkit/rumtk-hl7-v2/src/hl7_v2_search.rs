
use std::ops::{Index, IndexMut};
use std::collections::hash_map::{HashMap};
use rumtk_core::strings::{RUMString, format_compact, UTFStringExtensions,
                          RUMStringConversions};

/**************************** Globals **************************************/
static mut search_cache: HashMap<&str, V2FindComponent> = HashMap::with_capacity();

/**************************** Constants**************************************/

/**************************** Types *****************************************/

enum V2ComponentSearchType {
    DEFAULT,

}

pub struct V2FindComponent {
    segment: RUMString,
    segment_group: isize,
    field_group: isize,
    field: isize,
    component: isize
}

impl V2FindComponent {
    pub fn compile(expr: &str) -> V2FindComponent {

    }

    fn expr_type(expr: &str) -> Option<V2ComponentSearchType> {

    }
}

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
