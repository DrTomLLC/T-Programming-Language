//! Collections for T-Lang
//! Wrapper around Rust's std collections for now

// Re-export standard collections
pub use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, VecDeque, LinkedList};
pub use std::vec::Vec;

/// Create a new vector
pub fn vec<T>() -> Vec<T> {
    Vec::new()
}

/// Create a new hash map
pub fn hash_map<K, V>() -> HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    HashMap::new()
}

/// Create a new hash set
pub fn hash_set<T>() -> HashSet<T>
where
    T: std::hash::Hash + Eq,
{
    HashSet::new()
}

/// Create a vector from a slice
pub fn vec_from_slice<T: Clone>(slice: &[T]) -> Vec<T> {
    slice.to_vec()
}

/// Get the length of any collection with a len() method
pub trait Len {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<K, V> Len for HashMap<K, V> {
    fn len(&self) -> usize {
        HashMap::len(self)
    }
}

impl<T> Len for HashSet<T> {
    fn len(&self) -> usize {
        HashSet::len(self)
    }
}