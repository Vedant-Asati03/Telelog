//! Context management for structured logging with automatic cleanup.
//!
//! Contexts provide a way to add key-value pairs that are automatically included
//! in all log messages. Use [`ContextGuard`] for automatic cleanup when scope ends.
//!
//! # Examples
//!
//! ```
//! use telelog::Logger;
//!
//! let logger = Logger::new("app");
//!
//! // Manual context management
//! logger.add_context("user_id", "12345");
//! logger.info("Processing request");  // includes user_id
//! logger.clear_context();
//!
//! // Automatic cleanup with guard
//! {
//!     let _guard = logger.with_context("request_id", "abc");
//!     logger.info("Inside request");  // includes request_id
//! } // request_id automatically removed
//! ```

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// A collection of key-value pairs that are included in log messages.
#[derive(Debug, Clone)]
pub struct Context {
    data: HashMap<String, String>,
}

impl Context {
    /// Creates an empty context.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the context.
    pub fn add(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Removes a key from the context.
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// Removes all key-value pairs from the context.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Gets a value from the context by key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Returns `true` if the context contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns an iterator over the key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.data.iter()
    }

    /// Returns the number of key-value pairs in the context.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the context is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that automatically removes a context key when dropped.
///
/// This guard ensures that temporary context is cleaned up when it goes out of scope,
/// preventing context pollution across different operations.
pub struct ContextGuard {
    key: String,
    context: Arc<RwLock<Context>>,
}

impl ContextGuard {
    /// Creates a new context guard for the specified key.
    pub fn new(key: String, context: Arc<RwLock<Context>>) -> Self {
        Self { key, context }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        self.context.write().remove(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_operations() {
        let mut context = Context::new();

        assert!(context.is_empty());
        assert_eq!(context.len(), 0);

        context.add("key1", "value1");
        context.add("key2", "value2");

        assert!(!context.is_empty());
        assert_eq!(context.len(), 2);
        assert_eq!(context.get("key1"), Some(&"value1".to_string()));
        assert!(context.contains_key("key1"));

        context.remove("key1");
        assert_eq!(context.len(), 1);
        assert!(!context.contains_key("key1"));

        context.clear();
        assert!(context.is_empty());
    }

    #[test]
    fn test_context_iteration() {
        let mut context = Context::new();
        context.add("key1", "value1");
        context.add("key2", "value2");

        let mut count = 0;
        for (key, value) in context.iter() {
            assert!(key == "key1" || key == "key2");
            assert!(value == "value1" || value == "value2");
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_context_guard() {
        let context = Arc::new(RwLock::new(Context::new()));

        {
            context.write().add("temp_key", "temp_value");
            let _guard = ContextGuard::new("temp_key".to_string(), Arc::clone(&context));

            assert!(context.read().contains_key("temp_key"));
        } // guard is dropped here

        assert!(!context.read().contains_key("temp_key"));
    }
}
