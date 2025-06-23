//! Fundamental collection types for T-Lang.
//! For now, re-export or wrap Rust’s own; you can replace with custom structures later.

use std::collections::HashMap as StdHashMap;

/// A dynamically-sized, growable vector.
pub type Vec<T> = std::vec::Vec<T>;

/// A hash map from one type to another.
pub struct HashMap<K, V> {
    inner: StdHashMap<K, V>,
}

impl<K: std::hash::Hash + Eq, V> HashMap<K, V> {
    /// Create a new, empty `HashMap`.
    pub fn new() -> Self {
        Self { inner: StdHashMap::new() }
    }

    /// Insert a value into the map, returning the old value (if any).
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Get a reference to a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Remove a key from the map, returning the value if it existed.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    /// Number of elements in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// Future collection types (String utilities, Set, etc.) go here.
