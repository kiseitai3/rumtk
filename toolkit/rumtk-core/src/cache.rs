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


use core::hash::Hash;
use std::sync::Arc;
pub use ahash::AHashMap;
pub use once_cell::unsync::Lazy;
pub use std::sync::Mutex;
/**************************** Constants**************************************/
pub const DEFAULT_CACHE_PAGE_SIZE: usize = 10; /// I don't think most scenarios will need more than 10 items worth of memory pre-allocated at a time.
/**************************** Caches ****************************************/

/**************************** Types *****************************************/
///
/// Generic Cache store object. One use case will be to use a search string as the key and store
/// the search parsing object here.
///
pub type RUMCache<K, V> = AHashMap<K, V>;
pub type LazyRUMCache<K, V> = Lazy<Arc<RUMCache<K, V>>>;

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
pub const fn new_cache<K, V>() -> LazyRUMCache<K, V> {
    LazyRUMCache::new(|| { Arc::new(RUMCache::with_capacity(DEFAULT_CACHE_PAGE_SIZE)) })
}

pub fn get_or_set_from_cache<K, V, F>(cache: &'static mut LazyRUMCache<K, V>, expr: &K, new_fn: F) -> &'static V
where
    K: Hash + Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V
{
    if !cache.contains_key(expr) {
        let mut cache_ref = Arc::get_mut(cache).unwrap();
        cache_ref.insert(expr.clone(), new_fn(expr).clone());
    }
    cache.get(expr).unwrap()
}


