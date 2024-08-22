
use core::hash::Hash;
pub use ahash::AHashMap;
pub use once_cell::unsync::Lazy;
pub use std::sync::Mutex;
/**************************** Constants**************************************/

/**************************** Caches ****************************************/

/**************************** Types *****************************************/
///
/// Generic Cache store object. One use case will be to use a search string as the key and store
/// the search parsing object here.
///
pub type RUMCache<K, V> = AHashMap<K, V>;
pub type LazyRUMCache<K, V> = Lazy<RUMCache<K, V>>;

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
pub const fn new_cache<K, V>() -> LazyRUMCache<K, V> {
    LazyRUMCache::new(|| { RUMCache::default() })
}

pub fn get_or_set_from_cache<K, V, F>(cache: &'static mut LazyRUMCache<K, V>, expr: &K, new_fn: F) -> V
where
    K: Hash + std::cmp::Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V
{
    unsafe {
        if cache.contains_key(expr) {
            cache.get(expr).unwrap().clone()
        } else {
            cache.insert(expr.clone(), new_fn(expr));
            cache.get(expr).unwrap().clone()
        }
    }
}


