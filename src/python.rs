//! Python bindings for telelog

#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::Logger as RustLogger;

#[cfg(feature = "python")]
use crate::visualization::{ChartConfig, ChartType, Direction, MermaidGenerator};

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

    /// Log with structured data
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

    /// Create a performance profiling context manager
    fn profile(&self, operation: &str) -> ProfileContext {
        ProfileContext {
            guard: Some(self.inner.profile(operation)),
        }
    }

    /// Create a component tracking context manager
    fn track_component(&self, name: &str) -> ComponentContext {
        ComponentContext {
            guard: Some(self.inner.track_component(name)),
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

#[cfg(feature = "python")]
#[pyclass]
pub struct ProfileContext {
    guard: Option<crate::ProfileGuard>,
}

#[cfg(feature = "python")]
#[pyclass]
pub struct ComponentContext {
    guard: Option<crate::component::ComponentGuard>,
}

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

    // Theme support will be added in future versions
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
        // Drop the guard to trigger profiling log
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
        // Drop the guard to end component tracking
        self.guard.take();
        Ok(false)
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn create_logger(name: &str) -> PyResult<Logger> {
    Ok(Logger::new(name))
}

#[cfg(feature = "python")]
#[pymodule]
#[pyo3(name = "telelog")]
fn telelog_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Logger>()?;
    m.add_class::<ProfileContext>()?;
    m.add_class::<ComponentContext>()?;
    m.add_class::<VisualizationConfig>()?;
    m.add("__version__", crate::VERSION)?;
    m.add_function(wrap_pyfunction!(create_logger, m)?)?;

    Ok(())
}

// For non-Python builds, provide empty module
#[cfg(not(feature = "python"))]
pub struct Logger;
