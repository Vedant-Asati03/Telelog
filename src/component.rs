//! Component tracking for hierarchical visualization and performance analysis.
//!
//! This module provides utilities for tracking component execution, including:
//! - Hierarchical parent-child relationships
//! - Automatic timing and status tracking
//! - RAII guards for automatic lifecycle management
//! - Optional system resource monitoring
//!
//! # Examples
//!
//! ```
//! use telelog::ComponentTracker;
//! use std::sync::Arc;
//!
//! let tracker = Arc::new(ComponentTracker::new());
//! let _guard = telelog::ComponentGuard::new("operation", tracker);
//! // Component is automatically tracked and completed when guard drops
//! ```

use crate::level::LogLevel;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "system-monitor")]
use crate::monitor::SystemMonitor;

/// Status of a tracked component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// Component is currently executing
    Running,
    /// Component completed successfully
    Success,
    /// Component failed with an error message
    Failed(String),
    /// Component was cancelled before completion
    Cancelled,
}

/// Metadata associated with a tracked component.
///
/// Includes custom key-value pairs, memory usage, messages, and log levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub custom: HashMap<String, String>,
    pub memory_bytes: Option<u64>,
    pub message: Option<String>,
    pub level: LogLevel,
}

impl ComponentMetadata {
    /// Creates an empty metadata instance.
    pub fn new() -> Self {
        Self {
            custom: HashMap::new(),
            memory_bytes: None,
            message: None,
            level: LogLevel::Info,
        }
    }

    /// Adds a custom key-value pair to the metadata.
    pub fn with_custom(mut self, key: &str, value: &str) -> Self {
        self.custom.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the memory usage in bytes.
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Sets a message associated with the component.
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Sets the log level for the component.
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

/// A tracked component with timing, status, and hierarchical relationships.
///
/// Components automatically track their execution time and can be organized
/// in parent-child hierarchies for visualization.
#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub status: ComponentStatus,
    pub metadata: ComponentMetadata,
}

/// Serializable representation of a component for export.
///
/// Unlike [`Component`], this uses duration in milliseconds instead of `Instant`
/// for portability across serialization boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableComponent {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub duration_ms: Option<f64>,
    pub status: ComponentStatus,
    pub metadata: ComponentMetadata,
}

impl Component {
    /// Creates a new component with the given ID, name, and optional parent.
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

    /// Returns the duration of component execution, if completed.
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }

    /// Marks the component as completed with the given status.
    pub fn complete(&mut self, status: ComponentStatus) {
        self.end_time = Some(Instant::now());
        self.status = status;
    }

    /// Returns `true` if the component is currently running.
    pub fn is_running(&self) -> bool {
        matches!(self.status, ComponentStatus::Running)
    }

    /// Converts this component to a serializable representation.
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

/// Thread-safe tracker for managing component hierarchies.
///
/// Maintains a stack-based context for automatic parent-child relationships
/// and provides methods to query component trees.
#[derive(Debug)]
pub struct ComponentTracker {
    components: RwLock<HashMap<String, Component>>,
    current_stack: RwLock<Vec<String>>,
    next_id: RwLock<u64>,
}

impl ComponentTracker {
    /// Creates a new empty component tracker.
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            current_stack: RwLock::new(Vec::new()),
            next_id: RwLock::new(0),
        }
    }

    fn generate_id(&self) -> String {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        format!("comp_{}", id)
    }

    /// Starts tracking a new component.
    ///
    /// The component is automatically added as a child of the current component
    /// on the stack, if any. Returns the generated component ID.
    pub fn start_component(&self, name: &str) -> String {
        let id = self.generate_id();
        let parent_id = self.current_stack.read().last().cloned();

        let component = Component::new(id.clone(), name.to_string(), parent_id.clone());

        if let Some(parent_id) = &parent_id {
            if let Some(parent) = self.components.write().get_mut(parent_id) {
                parent.children.push(id.clone());
            }
        }

        self.components.write().insert(id.clone(), component);

        self.current_stack.write().push(id.clone());

        id
    }

    /// Ends a component and sets its status.
    ///
    /// # Errors
    ///
    /// Returns an error if no component with the given ID exists.
    pub fn end_component(&self, id: &str, status: ComponentStatus) -> Result<(), String> {
        let mut components = self.components.write();
        let mut stack = self.current_stack.write();

        if let Some(pos) = stack.iter().position(|x| x == id) {
            stack.remove(pos);
        }

        if let Some(component) = components.get_mut(id) {
            component.complete(status);
            Ok(())
        } else {
            Err(format!("Component with ID '{}' not found", id))
        }
    }

    /// Updates the metadata for a component.
    ///
    /// # Errors
    ///
    /// Returns an error if no component with the given ID exists.
    pub fn update_metadata(&self, id: &str, metadata: ComponentMetadata) -> Result<(), String> {
        let mut components = self.components.write();
        if let Some(component) = components.get_mut(id) {
            component.metadata = metadata;
            Ok(())
        } else {
            Err(format!("Component with ID '{}' not found", id))
        }
    }

    /// Returns a copy of all tracked components.
    pub fn get_components(&self) -> HashMap<String, Component> {
        self.components.read().clone()
    }

    /// Returns all root components (those without a parent).
    pub fn get_root_components(&self) -> Vec<Component> {
        self.components
            .read()
            .values()
            .filter(|c| c.parent_id.is_none())
            .cloned()
            .collect()
    }

    /// Returns all child components of the specified parent.
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

    /// Clears all tracked components and resets internal state.
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

/// RAII guard that automatically tracks and completes a component.
///
/// When dropped, the component is marked as successful. Use explicit completion
/// methods like [`complete_success`](Self::complete_success) or
/// [`complete_failure`](Self::complete_failure) for specific status.
///
/// # Examples
///
/// ```
/// use telelog::{ComponentTracker, ComponentGuard};
/// use std::sync::Arc;
///
/// let tracker = Arc::new(ComponentTracker::new());
/// {
///     let guard = ComponentGuard::new("my_operation", tracker.clone());
///     // Do work...
/// } // Component automatically marked as Success on drop
/// ```
pub struct ComponentGuard {
    id: String,
    tracker: Arc<ComponentTracker>,
    #[cfg(feature = "system-monitor")]
    system_monitor: Option<Arc<RwLock<SystemMonitor>>>,
    start_memory: Option<u64>,
}

impl ComponentGuard {
    /// Creates a new component guard with the given name.
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

    /// Creates a new component guard with system monitoring enabled.
    ///
    /// Tracks memory usage delta from component start to end.
    pub fn new_with_monitor(
        name: &str,
        tracker: Arc<ComponentTracker>,
        monitor: Arc<RwLock<SystemMonitor>>,
    ) -> Self {
        let id = tracker.start_component(name);

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

    /// Returns the component ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Updates the complete metadata for this component.
    pub fn update_metadata(&self, metadata: ComponentMetadata) -> Result<(), String> {
        self.tracker.update_metadata(&self.id, metadata)
    }

    /// Adds a single key-value pair to the component's metadata.
    pub fn add_metadata(&self, key: &str, value: &str) -> Result<(), String> {
        let components = self.tracker.components.read();
        if let Some(component) = components.get(&self.id) {
            let mut metadata = component.metadata.clone();
            metadata.custom.insert(key.to_string(), value.to_string());
            drop(components);
            self.tracker.update_metadata(&self.id, metadata)
        } else {
            Err(format!("Component with ID '{}' not found", self.id))
        }
    }

    /// Updates memory usage from the current system state.
    ///
    /// Calculates memory delta since component creation if available.
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

    /// Marks the component as successfully completed and prevents drop behavior.
    pub fn complete_success(self) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Success);
        std::mem::forget(self);
    }

    /// Marks the component as failed with an error message and prevents drop behavior.
    pub fn complete_failure(self, error: &str) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Failed(error.to_string()));
        std::mem::forget(self);
    }

    /// Marks the component as cancelled and prevents drop behavior.
    pub fn complete_cancelled(self) {
        let _ = self
            .tracker
            .end_component(&self.id, ComponentStatus::Cancelled);
        std::mem::forget(self);
    }
}

impl Drop for ComponentGuard {
    fn drop(&mut self) {
        #[cfg(feature = "system-monitor")]
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
                let _ = self.tracker.update_metadata(&self.id, metadata);
            }
        }

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
