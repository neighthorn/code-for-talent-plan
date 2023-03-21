#![deny(missing_docs)]
//! This is a simple key-value store

use std::collections::HashMap;

/// 'KvStore' is a Hashmap that can store key-value pairs
///
/// Example
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), Some("value1".to_owned()));
/// ```
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// create an empty KvStore
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }
    /// insert a key-value pair in KvStore
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }
    /// get the value for the given key
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }
    /// reomve the key-value pair with given key
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
