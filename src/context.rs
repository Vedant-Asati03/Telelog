//! Context management for structured logging

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Context for adding structured data to log messages
#[derive(Debug, Clone)]
pub struct Context {
    data: HashMap<String, String>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Add a key-value pair to the context
    pub fn add(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Remove a key from the context
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// Clear all context data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get a value from the context
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Check if the context contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.data.iter()
    }

    /// Get the number of context entries
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// A guard that automatically removes context when dropped
pub struct ContextGuard {
    key: String,
    context: Arc<RwLock<Context>>,
}

impl ContextGuard {
    /// Create a new context guard
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
