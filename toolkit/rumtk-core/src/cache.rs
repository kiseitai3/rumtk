
use std::ops::{Index, IndexMut};
use ahash::AHashMap;
use crate::strings::{RUMString, format_compact, UTFStringExtensions,
                     RUMStringConversions};

/**************************** Constants**************************************/

/**************************** Types *****************************************/

///
/// Generic Cache store object. One use case will be to use a search string as the key and store
/// the search parsing object here.
///
pub struct RUMCache<K, V> {
    limit: usize,
    cache: AHashMap<K, V>
}

impl<K, V> RUMCache<K, V> {
    pub fn new(capacity: usize, max_size: usize) -> RUMCache<K, V> {
        RUMCache{ limit: max_size, cache: AHashMap::with_capacity(capacity) }
    }

    pub fn push(&self, key: &K, val: &V) {
        self.free_a_slot();
        self.cache[key] = val;
    }

    pub fn pop(&self, key: &K) {
        self.cache.remove(key);
    }

    fn free_a_slot(&self) {
        if self.cache.len() > self.limit {
            match self.get_a_key() {
                Some(k) => self.pop(k),
                None => ()
            }
        }
    }

    pub fn get(&self, key: &K) -> &V {
        &self.cache[key]
    }

    pub fn get_mut(&self, key: &K) -> &mut V {
        &mut self.cache[key]
    }

    fn get_a_key(&self) -> Option<&K> {
        for key in self.cache.keys() {
            return Some(key);
        }
        None
    }
}

/**************************** Traits ****************************************/


impl<K, V> Index<K> for RUMCache<K, V> {
    type Output = V;
    fn index(&self, indx: &K) -> &Self::Output {
        self.get(indx)
    }
}

impl<K, V> IndexMut<K> for RUMCache<K, V> {
    fn index_mut(&mut self, indx: &K) -> &mut V {
        self.get_mut(indx)
    }
}

/**************************** Helpers ***************************************/
