//! Python bindings for telelog

#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::Logger as RustLogger;

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
#[pymethods]
impl ProfileContext {
    fn __enter__(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        // Drop the guard to trigger profiling log
        self.guard.take();
        Ok(false)
    }
}

#[cfg(feature = "python")]
#[pymodule]
#[pyo3(name = "telelog")]
fn telelog_native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Logger>()?;
    m.add_class::<ProfileContext>()?;
    m.add("__version__", crate::VERSION)?;

    #[pyfn(m)]
    fn create_logger(_py: Python, name: &str) -> PyResult<Logger> {
        Ok(Logger::new(name))
    }

    Ok(())
}

// For non-Python builds, provide empty module
#[cfg(not(feature = "python"))]
pub struct Logger;
