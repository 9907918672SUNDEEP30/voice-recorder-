//! # Execution Context
//!
//! Key-value shared runtime memory passed sequentially between steps.
//! Application-agnostic and free of third-party dependencies.

use std::collections::HashMap;

/// Runtime memory for a single execution run.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub run_id: String,
    store: HashMap<String, String>,
}

impl ExecutionContext {
    /// Creates a new execution context with a given run ID.
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            store: HashMap::new(),
        }
    }

    /// Retrieves a value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.store.get(key).map(|s| s.as_str())
    }

    /// Stores a key-value pair.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.store.insert(key.into(), value.into());
    }

    /// Removes a key from the context.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.store.remove(key)
    }

    /// Checks if a key exists in context.
    pub fn contains_key(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    /// Returns the number of entries in the context.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns true if context is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Iterates over all key-value pairs.
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, String> {
        self.store.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_stores_and_retrieves_values() {
        let mut ctx = ExecutionContext::new("run_test_1");
        assert_eq!(ctx.run_id, "run_test_1");
        assert!(ctx.is_empty());

        ctx.set("input", "hello world");
        assert_eq!(ctx.get("input"), Some("hello world"));
        assert_eq!(ctx.len(), 1);

        assert_eq!(ctx.remove("input"), Some("hello world".to_string()));
        assert_eq!(ctx.get("input"), None);
    }
}
