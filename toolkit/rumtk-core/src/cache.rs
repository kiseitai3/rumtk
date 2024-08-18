
pub use ahash::AHashMap;
pub use once_cell::unsync::Lazy;
pub use std::sync::Mutex;

/**************************** Constants**************************************/

/**************************** Types *****************************************/
///
/// Generic Cache store object. One use case will be to use a search string as the key and store
/// the search parsing object here.
///
pub type RUMCache<K, V> = Lazy<AHashMap<K, V>>;

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/

pub const fn new_cache<K, V>() -> RUMCache<K, V> {
    RUMCache::new(|| { AHashMap::default() })
}
