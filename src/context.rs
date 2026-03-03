use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Context {
    pub(crate) data: RwLock<HashMap<String, String>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
    pub fn add(&self, key: &str, value: &str) {
        self.data.write().insert(key.to_string(), value.to_string());
    }
    pub fn remove(&self, key: &str) {
        self.data.write().remove(key);
    }
    pub fn clear(&self) {
        self.data.write().clear();
    }
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }
    pub fn len(&self) -> usize {
        self.data.read().len()
    }
}

pub struct ContextGuard {
    key: String,
    context: Arc<Context>,
}

impl ContextGuard {
    pub fn new(key: String, context: Arc<Context>) -> Self {
        Self { key, context }
    }
}
impl Drop for ContextGuard {
    fn drop(&mut self) {
        self.context.remove(&self.key);
    }
}
