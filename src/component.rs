//! Component tracking for visualization

use crate::level::LogLevel;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "system-monitor")]
use crate::monitor::SystemMonitor;

/// Status of a component execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// Component is currently running
    Running,
    /// Component completed successfully
    Success,
    /// Component failed with error
    Failed(String),
    /// Component was cancelled/interrupted
    Cancelled,
}

/// Metadata for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Custom key-value metadata
    pub custom: HashMap<String, String>,
    /// Memory usage in bytes (if available)
    pub memory_bytes: Option<u64>,
    /// Log message associated with this component
    pub message: Option<String>,
    /// Log level of the component
    pub level: LogLevel,
}

impl ComponentMetadata {
    pub fn new() -> Self {
        Self {
            custom: HashMap::new(),
            memory_bytes: None,
            message: None,
            level: LogLevel::Info,
        }
    }

    pub fn with_custom(mut self, key: &str, value: &str) -> Self {
        self.custom.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// A tracked component with timing and metadata
#[derive(Debug, Clone)]
pub struct Component {
    /// Unique identifier for this component
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Parent component ID (if any)
    pub parent_id: Option<String>,
    /// Child component IDs
    pub children: Vec<String>,
    /// When the component started
    pub start_time: Instant,
    /// When the component ended (if completed)
    pub end_time: Option<Instant>,
    /// Current status
    pub status: ComponentStatus,
    /// Additional metadata
    pub metadata: ComponentMetadata,
}

/// Serializable version of Component for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableComponent {
    /// Unique identifier for this component
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Parent component ID (if any)
    pub parent_id: Option<String>,
    /// Child component IDs
    pub children: Vec<String>,
    /// Duration in milliseconds (if completed)
    pub duration_ms: Option<f64>,
    /// Current status
    pub status: ComponentStatus,
    /// Additional metadata
    pub metadata: ComponentMetadata,
}

impl Component {
    pub fn new(id: String, name: String, parent_id: Option<String>) -> Self {
        Self {
            id,
            name,
            parent_id,
            children: Vec::new(),
            start_time: Instant::now(),
            end_time: None,
            status: ComponentStatus::Running,
            metadata: ComponentMetadata::new(),
        }
    }

    /// Get the duration of this component (if completed)
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }

    /// Mark component as completed with status
    pub fn complete(&mut self, status: ComponentStatus) {
        self.end_time = Some(Instant::now());
        self.status = status;
    }

    /// Check if component is still running
    pub fn is_running(&self) -> bool {
        matches!(self.status, ComponentStatus::Running)
    }

    /// Convert to serializable version
    pub fn to_serializable(&self) -> SerializableComponent {
        SerializableComponent {
            id: self.id.clone(),
            name: self.name.clone(),
            parent_id: self.parent_id.clone(),
            children: self.children.clone(),
            duration_ms: self.duration().map(|d| d.as_secs_f64() * 1000.0),
            status: self.status.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Component tracking system
#[derive(Debug)]
pub struct ComponentTracker {
    /// All tracked components
    components: RwLock<HashMap<String, Component>>,
    /// Current component stack (for automatic parent-child relationships)
    current_stack: RwLock<Vec<String>>,
    /// Counter for generating unique IDs
    next_id: RwLock<u64>,
}

impl ComponentTracker {
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            current_stack: RwLock::new(Vec::new()),
            next_id: RwLock::new(0),
        }
    }

    /// Generate a unique component ID
    fn generate_id(&self) -> String {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        format!("comp_{}", id)
    }

    /// Start tracking a new component
    pub fn start_component(&self, name: &str) -> String {
        let id = self.generate_id();
        let parent_id = self.current_stack.read().last().cloned();

        let component = Component::new(id.clone(), name.to_string(), parent_id.clone());

        // Add to parent's children list
        if let Some(parent_id) = &parent_id {
            if let Some(parent) = self.components.write().get_mut(parent_id) {
                parent.children.push(id.clone());
            }
        }

        // Add to components map
        self.components.write().insert(id.clone(), component);

        // Push to current stack
        self.current_stack.write().push(id.clone());

        id
    }

    /// End tracking a component
    pub fn end_component(&self, id: &str, status: ComponentStatus) -> Result<(), String> {
        let mut components = self.components.write();
        let mut stack = self.current_stack.write();

        // Remove from stack
        if let Some(pos) = stack.iter().position(|x| x == id) {
            stack.remove(pos);
        }

        // Update component
        if let Some(component) = components.get_mut(id) {
            component.complete(status);
            Ok(())
        } else {
            Err(format!("Component with ID '{}' not found", id))
        }
    }

    /// Update component metadata
    pub fn update_metadata(&self, id: &str, metadata: ComponentMetadata) -> Result<(), String> {
        let mut components = self.components.write();
        if let Some(component) = components.get_mut(id) {
            component.metadata = metadata;
            Ok(())
        } else {
            Err(format!("Component with ID '{}' not found", id))
        }
    }

    /// Get all components
    pub fn get_components(&self) -> HashMap<String, Component> {
        self.components.read().clone()
    }

    /// Get root components (components with no parent)
    pub fn get_root_components(&self) -> Vec<Component> {
        self.components
            .read()
            .values()
            .filter(|c| c.parent_id.is_none())
            .cloned()
            .collect()
    }

    /// Get children of a specific component
    pub fn get_children(&self, parent_id: &str) -> Vec<Component> {
        let components = self.components.read();
        if let Some(parent) = components.get(parent_id) {
            parent
                .children
                .iter()
                .filter_map(|child_id| components.get(child_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all tracked components
    pub fn clear(&self) {
        self.components.write().clear();
        self.current_stack.write().clear();
        *self.next_id.write() = 0;
    }
}

impl Default for ComponentTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for automatic component tracking
pub struct ComponentGuard {
    id: String,
    tracker: Arc<ComponentTracker>,
    #[cfg(feature = "system-monitor")]
    system_monitor: Option<Arc<RwLock<SystemMonitor>>>,
    start_memory: Option<u64>,
}

impl ComponentGuard {
    pub fn new(name: &str, tracker: Arc<ComponentTracker>) -> Self {
        let id = tracker.start_component(name);
        Self {
            id,
            tracker,
            #[cfg(feature = "system-monitor")]
            system_monitor: None,
            start_memory: None,
        }
    }

    /// Create a new component guard with system monitoring
    #[cfg(feature = "system-monitor")]
    pub fn new_with_monitor(
        name: &str,
        tracker: Arc<ComponentTracker>,
        monitor: Arc<RwLock<SystemMonitor>>,
    ) -> Self {
        let id = tracker.start_component(name);

        // Capture initial memory usage
        let start_memory = {
            let mut monitor_guard = monitor.write();
            monitor_guard.refresh();
            monitor_guard.process_memory()
        };

        Self {
            id,
            tracker,
            system_monitor: Some(monitor),
            start_memory,
        }
    }

    /// Get the component ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Update metadata for this component
    pub fn update_metadata(&self, metadata: ComponentMetadata) -> Result<(), String> {
        self.tracker.update_metadata(&self.id, metadata)
    }

    /// Add custom metadata
    pub fn add_metadata(&self, key: &str, value: &str) -> Result<(), String> {
        let components = self.tracker.components.read();
        if let Some(component) = components.get(&self.id) {
            let mut metadata = component.metadata.clone();
            metadata.custom.insert(key.to_string(), value.to_string());
            drop(components); // Release read lock
            self.tracker.update_metadata(&self.id, metadata)
        } else {
            Err(format!("Component with ID '{}' not found", self.id))
        }
    }

    /// Update memory usage from current system state
    #[cfg(feature = "system-monitor")]
    pub fn update_memory_usage(&self) -> Result<(), String> {
        if let Some(monitor) = &self.system_monitor {
            let mut monitor_guard = monitor.write();
            monitor_guard.refresh();
            if let Some(current_memory) = monitor_guard.process_memory() {
                let memory_to_store = if let Some(start_mem) = self.start_memory {
                    if current_memory > start_mem {
                        current_memory - start_mem
                    } else {
                        current_memory
                    }
                } else {
                    current_memory
                };

                let metadata = ComponentMetadata::new().with_memory(memory_to_store);
                self.tracker.update_metadata(&self.id, metadata)
            } else {
                Err("Failed to get current memory usage".to_string())
            }
        } else {
            Err("System monitor not available".to_string())
        }
    }

    /// Complete with success status
    pub fn complete_success(self) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Success);
        std::mem::forget(self); // Prevent drop from running
    }

    /// Complete with failure status
    pub fn complete_failure(self, error: &str) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Failed(error.to_string()));
        std::mem::forget(self); // Prevent drop from running
    }

    /// Complete with cancelled status
    pub fn complete_cancelled(self) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Cancelled);
        std::mem::forget(self); // Prevent drop from running
    }
}

impl Drop for ComponentGuard {
    fn drop(&mut self) {
        // Capture final memory usage if system monitor is available
        #[cfg(feature = "system-monitor")]
        if let Some(monitor) = &self.system_monitor {
            let mut monitor_guard = monitor.write();
            monitor_guard.refresh();
            if let Some(current_memory) = monitor_guard.process_memory() {
                // Calculate memory delta if we have start memory
                let memory_to_store = if let Some(start_mem) = self.start_memory {
                    if current_memory > start_mem {
                        current_memory - start_mem // Memory allocated by this component
                    } else {
                        current_memory // Just store current memory if it decreased
                    }
                } else {
                    current_memory
                };

                // Update component metadata with memory usage
                let metadata = ComponentMetadata::new().with_memory(memory_to_store);
                let _ = self.tracker.update_metadata(&self.id, metadata);
            }
        }

        // Auto-complete with success if not explicitly completed
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let tracker = ComponentTracker::new();
        let id = tracker.start_component("test_component");

        let components = tracker.get_components();
        assert_eq!(components.len(), 1);
        assert!(components.contains_key(&id));

        let component = &components[&id];
        assert_eq!(component.name, "test_component");
        assert!(component.is_running());
    }

    #[test]
    fn test_parent_child_relationship() {
        let tracker = ComponentTracker::new();

        let parent_id = tracker.start_component("parent");
        let child_id = tracker.start_component("child");

        let components = tracker.get_components();
        let parent = &components[&parent_id];
        let child = &components[&child_id];

        assert_eq!(child.parent_id, Some(parent_id.clone()));
        assert!(parent.children.contains(&child_id));
    }

    #[test]
    fn test_component_guard() {
        let tracker = Arc::new(ComponentTracker::new());

        {
            let _guard = ComponentGuard::new("test", tracker.clone());
            let components = tracker.get_components();
            assert_eq!(components.len(), 1);
        }

        // Component should be completed after guard drops
        let components = tracker.get_components();
        let component = components.values().next().unwrap();
        assert_eq!(component.status, ComponentStatus::Success);
    }

    #[test]
    fn test_metadata_updates() {
        let tracker = ComponentTracker::new();
        let id = tracker.start_component("test");

        let metadata = ComponentMetadata::new()
            .with_custom("key", "value")
            .with_memory(1024)
            .with_message("Test message");

        tracker.update_metadata(&id, metadata).unwrap();

        let components = tracker.get_components();
        let component = &components[&id];
        assert_eq!(
            component.metadata.custom.get("key"),
            Some(&"value".to_string())
        );
        assert_eq!(component.metadata.memory_bytes, Some(1024));
        assert_eq!(component.metadata.message, Some("Test message".to_string()));
    }
}
