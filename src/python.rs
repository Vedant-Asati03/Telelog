//! Python bindings for telelog via PyO3.
//!
//! Provides Python-friendly wrappers around the core Rust functionality,
//! including logging, configuration, profiling, and visualization.
//!
//! Requires the `python` feature to be enabled.

#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use std::collections::HashMap;

#[cfg(feature = "python")]
use crate::Logger as RustLogger;

#[cfg(feature = "python")]
use crate::config::Config as RustConfig;

#[cfg(feature = "python")]
use crate::level::LogLevel;

#[cfg(feature = "python")]
use crate::visualization::{ChartConfig, ChartType, Direction, MermaidGenerator};

/// Python-exposed configuration for telelog logger.
#[cfg(feature = "python")]
#[pyclass]
pub struct Config {
    inner: RustConfig,
}

#[cfg(feature = "python")]
#[pymethods]
impl Config {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustConfig::new(),
        }
    }

    /// Set minimum log level (debug, info, warning, error, critical)
    fn with_min_level(&mut self, level: &str) -> PyResult<()> {
        let log_level = match level.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warning" | "warn" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "critical" | "crit" => LogLevel::Critical,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid log level: {}",
                    level
                )))
            }
        };
        self.inner = self.inner.clone().with_min_level(log_level);
        Ok(())
    }

    fn with_console_output(&mut self, enabled: bool) {
        self.inner = self.inner.clone().with_console_output(enabled);
    }

    fn with_file_output(&mut self, path: &str) {
        self.inner = self.inner.clone().with_file_output(path);
    }

    fn with_json_format(&mut self, enabled: bool) {
        self.inner = self.inner.clone().with_json_format(enabled);
    }

    fn with_colored_output(&mut self, enabled: bool) {
        self.inner = self.inner.clone().with_colored_output(enabled);
    }

    fn with_profiling(&mut self, enabled: bool) {
        self.inner = self.inner.clone().with_profiling(enabled);
    }

    fn with_buffering(&mut self, enabled: bool) {
        self.inner = self.inner.clone().with_buffering(enabled);
    }

    fn with_buffer_size(&mut self, size: usize) {
        self.inner = self.inner.clone().with_buffer_size(size);
    }
}

/// Python-exposed logger for structured logging and profiling.
#[cfg(feature = "python")]
#[pyclass]
pub struct Logger {
    inner: RustLogger,
}

#[cfg(feature = "python")]
#[pymethods]
impl Logger {
    #[new]
    fn new(name: &str) -> Self {
        Self {
            inner: RustLogger::new(name),
        }
    }

    #[staticmethod]
    fn with_config(name: &str, config: &Config) -> Self {
        Self {
            inner: RustLogger::with_config(name, config.inner.clone()),
        }
    }

    fn get_config(&self) -> Config {
        Config {
            inner: self.inner.get_config(),
        }
    }

    fn set_config(&self, config: &Config) {
        self.inner.set_config(config.inner.clone());
    }

    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    fn debug(&self, message: &str) {
        self.inner.debug(message);
    }

    fn info(&self, message: &str) {
        self.inner.info(message);
    }

    fn warning(&self, message: &str) {
        self.inner.warning(message);
    }

    fn error(&self, message: &str) {
        self.inner.error(message);
    }

    fn critical(&self, message: &str) {
        self.inner.critical(message);
    }

    fn debug_with(&self, message: &str, data: Vec<(String, String)>) {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.inner.debug_with(message, &data_refs);
    }

    fn info_with(&self, message: &str, data: Vec<(String, String)>) {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.inner.info_with(message, &data_refs);
    }

    fn warning_with(&self, message: &str, data: Vec<(String, String)>) {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.inner.warning_with(message, &data_refs);
    }

    fn error_with(&self, message: &str, data: Vec<(String, String)>) {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.inner.error_with(message, &data_refs);
    }

    fn critical_with(&self, message: &str, data: Vec<(String, String)>) {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.inner.critical_with(message, &data_refs);
    }

    fn log_with(&self, level: &str, message: &str, data: Vec<(String, String)>) -> PyResult<()> {
        let data_refs: Vec<(&str, &str)> =
            data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        match level.to_lowercase().as_str() {
            "debug" => self.inner.debug_with(message, &data_refs),
            "info" => self.inner.info_with(message, &data_refs),
            "warning" | "warn" => self.inner.warning_with(message, &data_refs),
            "error" => self.inner.error_with(message, &data_refs),
            "critical" | "crit" => self.inner.critical_with(message, &data_refs),
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid log level: {}",
                    level
                )))
            }
        }
        Ok(())
    }

    fn add_context(&self, key: &str, value: &str) {
        self.inner.add_context(key, value);
    }

    fn remove_context(&self, key: &str) {
        self.inner.remove_context(key);
    }

    fn clear_context(&self) {
        self.inner.clear_context();
    }

    /// Add temporary context that will be automatically removed when exiting the context manager.
    ///
    /// # Example
    /// ```python
    /// with logger.with_context("request_id", "12345"):
    ///     logger.info("Processing request")  # includes request_id
    /// # request_id is automatically removed here
    /// ```
    fn with_context(&self, key: &str, value: &str) -> ContextManager {
        self.inner.add_context(key, value);
        ContextManager {
            key: key.to_string(),
            logger: self.inner.clone(),
        }
    }

    fn profile(&self, operation: &str) -> ProfileContext {
        ProfileContext {
            guard: Some(self.inner.profile(operation)),
        }
    }

    fn track_component(&self, name: &str) -> ComponentContext {
        ComponentContext {
            guard: Some(self.inner.track_component(name)),
        }
    }

    fn get_component_tracker(&self) -> ComponentTrackerWrapper {
        ComponentTrackerWrapper {
            tracker: self.inner.component_tracker().clone(),
        }
    }

    /// Generate visualization diagram
    #[pyo3(signature = (chart_type, output_path = None))]
    fn generate_visualization(
        &self,
        chart_type: &str,
        output_path: Option<&str>,
    ) -> PyResult<String> {
        let chart_type = match chart_type.to_lowercase().as_str() {
            "flowchart" => ChartType::Flowchart,
            "timeline" => ChartType::Timeline,
            "gantt" => ChartType::Gantt,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid chart type: {}. Use 'flowchart', 'timeline', or 'gantt'",
                    chart_type
                )))
            }
        };

        let config = ChartConfig::new().with_chart_type(chart_type);
        let generator = MermaidGenerator::new(config);
        let tracker = self.inner.get_component_tracker();

        let diagram = generator.generate_diagram(&tracker).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Visualization generation failed: {}",
                e
            ))
        })?;

        if let Some(path) = output_path {
            std::fs::write(path, &diagram).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to write file: {}", e))
            })?;
        }

        Ok(diagram)
    }

    fn __str__(&self) -> String {
        format!("TelelogLogger({})", self.inner.name())
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

/// Wrapper for component tracker access from Python.
#[cfg(feature = "python")]
#[pyclass]
pub struct ComponentTrackerWrapper {
    tracker: std::sync::Arc<crate::component::ComponentTracker>,
}

#[cfg(feature = "python")]
#[pymethods]
impl ComponentTrackerWrapper {
    /// Get all tracked components as a list of dictionaries
    fn get_all_components(&self) -> Vec<HashMap<String, String>> {
        let components = self.tracker.get_components();
        components
            .values()
            .map(|c| {
                let mut map = HashMap::new();
                map.insert("id".to_string(), c.id.clone());
                map.insert("name".to_string(), c.name.clone());
                map.insert("status".to_string(), format!("{:?}", c.status));
                if let Some(parent_id) = &c.parent_id {
                    map.insert("parent_id".to_string(), parent_id.clone());
                }
                map.insert(
                    "duration_ms".to_string(),
                    c.end_time
                        .map(|end| end.duration_since(c.start_time).as_millis().to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                );
                map
            })
            .collect()
    }

    fn count(&self) -> usize {
        self.tracker.get_components().len()
    }
}

/// Python context manager for automatic context cleanup.
#[cfg(feature = "python")]
#[pyclass]
pub struct ContextManager {
    key: String,
    logger: RustLogger,
}

/// Python context manager for profiling operations.
#[cfg(feature = "python")]
#[pyclass]
pub struct ProfileContext {
    guard: Option<crate::ProfileGuard>,
}

/// Python context manager for component tracking.
#[cfg(feature = "python")]
#[pyclass]
pub struct ComponentContext {
    guard: Option<crate::component::ComponentGuard>,
}

/// Python-exposed visualization configuration.
#[cfg(feature = "python")]
#[pyclass]
pub struct VisualizationConfig {
    inner: ChartConfig,
}

#[cfg(feature = "python")]
#[pymethods]
impl VisualizationConfig {
    #[new]
    fn new() -> Self {
        Self {
            inner: ChartConfig::new(),
        }
    }

    fn with_chart_type(&mut self, chart_type: &str) -> PyResult<()> {
        let chart_type = match chart_type.to_lowercase().as_str() {
            "flowchart" => ChartType::Flowchart,
            "timeline" => ChartType::Timeline,
            "gantt" => ChartType::Gantt,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid chart type: {}",
                    chart_type
                )))
            }
        };
        self.inner = self.inner.clone().with_chart_type(chart_type);
        Ok(())
    }

    fn with_direction(&mut self, direction: &str) -> PyResult<()> {
        let direction = match direction.to_lowercase().as_str() {
            "topdown" | "td" | "tb" => Direction::TopDown,
            "bottomup" | "bu" | "bt" => Direction::BottomUp,
            "leftright" | "lr" => Direction::LeftRight,
            "rightleft" | "rl" => Direction::RightLeft,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid direction: {}",
                    direction
                )))
            }
        };
        self.inner = self.inner.clone().with_direction(direction);
        Ok(())
    }

    fn set_timing(&mut self, show_timing: bool) {
        self.inner.show_timing = show_timing;
    }

    fn set_memory(&mut self, show_memory: bool) {
        self.inner.show_memory = show_memory;
    }

    fn set_metadata(&mut self, show_metadata: bool) {
        self.inner.show_metadata = show_metadata;
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl ContextManager {
    fn __enter__(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<Py<PyAny>>,
        _exc_value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>,
    ) -> PyResult<bool> {
        self.logger.remove_context(&self.key);
        Ok(false)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl ProfileContext {
    fn __enter__(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<Py<PyAny>>,
        _exc_value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>,
    ) -> PyResult<bool> {
        self.guard.take();
        Ok(false)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl ComponentContext {
    fn __enter__(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<Py<PyAny>>,
        _exc_value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>,
    ) -> PyResult<bool> {
        self.guard.take();
        Ok(false)
    }
}

/// Creates a new logger instance (convenience function for Python).
#[cfg(feature = "python")]
#[pyfunction]
fn create_logger(name: &str) -> PyResult<Logger> {
    Ok(Logger::new(name))
}

/// Python module definition for telelog.
#[cfg(feature = "python")]
#[pymodule]
#[pyo3(name = "telelog")]
fn telelog_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Logger>()?;
    m.add_class::<Config>()?;
    m.add_class::<ContextManager>()?;
    m.add_class::<ProfileContext>()?;
    m.add_class::<ComponentContext>()?;
    m.add_class::<ComponentTrackerWrapper>()?;
    m.add_class::<VisualizationConfig>()?;
    m.add("__version__", crate::VERSION)?;
    m.add_function(wrap_pyfunction!(create_logger, m)?)?;

    Ok(())
}

/// Stub logger type when Python feature is disabled.
#[cfg(not(feature = "python"))]
pub struct Logger;
