
pub use ahash::AHashMap;

/**************************** Constants**************************************/

/**************************** Types *****************************************/

///
/// Generic Cache store object. One use case will be to use a search string as the key and store
/// the search parsing object here.
///
pub type RUMCache<K, V> = AHashMap<K, V>;

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
