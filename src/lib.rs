#![deny(missing_docs)]

//! Library for storing in memory string values under string keys.

use std::collections::HashMap;

/// The [`KvStore`] stores string key-value pairs.
///
/// Key-value pairs are stored in a [`HashMap`] in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_string(), "value".to_string());
///
/// let val = store.get("key".to_string());
/// assert_eq!(val, Some("value".to_string()));
///
/// store.remove("key".to_string());
///
/// let val = store.get("key".to_string());
/// assert_eq!(val, None);
/// ```
#[derive(Debug, Default)]
pub struct KvStore(HashMap<String, String>);

impl KvStore {
    /// Creates an empty `KvStore`
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Sets the given key to provided value.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    /// Gets the value of a given key.
    ///
    /// Returns [`None`] if the key does not exist.
    pub fn get(&self, key: String) -> Option<String> {
        self.0.get(&key).cloned()
    }

    /// Removes a given key.
    pub fn remove(&mut self, key: String) {
        self.0.remove(&key);
    }
}
