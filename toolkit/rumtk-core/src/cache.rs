
use std::ops::{Index, IndexMut};
use std::collections::hash_map::{HashMap};
use crate::strings::{RUMString, format_compact, UTFStringExtensions,
                     RUMStringConversions};

/**************************** Constants**************************************/

/**************************** Types *****************************************/

pub struct RUMCache<K, V> {
    limit: usize,
    cache: HashMap<K, V>
}

impl<K, V> RUMCache<K, V> {
    pub fn new(capacity: usize, max_size: usize) -> RUMCache<K, V> {
        RUMCache{ limit: max_size, cache: HashMap::with_capacity(capacity) }
    }

    pub fn push(&self, key: &K, val: &V) {

    }

    pub fn pop(&self, key: &K) {

    }

    pub fn get(&self, key: &K) -> &V {

    }
}

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
