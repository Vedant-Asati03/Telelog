from typing import List, Dict, Optional, Tuple, Any, Sequence, Literal

__version__: str

LogLevel = Literal["debug", "info", "warning", "warn", "error", "critical", "crit"]
ChartType = Literal["flowchart", "timeline", "gantt"]
Direction = Literal[
    "topdown", "td", "tb", "bottomup", "bu", "bt", "leftright", "lr", "rightleft", "rl"
]

class Config:
    """
    Configuration builder for the Logger.

    Allows chaining methods to build a configuration object fluently before passing it
    to a logger.

    Example:
        ```python
        import telelog as tl
        config = tl.Config().with_min_level("debug").with_console_output(True)
        logger = tl.Logger.with_config("my_app", config)
        ```
    """

    def __init__(self) -> None: ...
    def with_min_level(self, level: LogLevel) -> "Config":
        """
        Sets the minimum log level filter.

        Any log message with a severity lower than `level` will be ignored.

        Args:
            level: The minimum log level as a string (e.g., "debug", "info", "warning", "error").

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_console_output(self, enabled: bool) -> "Config":
        """
        Enables or disables logging to the terminal/console (stdout/stderr).

        Args:
            enabled: True to log to the console, False to silence console output.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_file_output(self, path: str) -> "Config":
        """
        Enables logging directly to a specific file.

        Args:
            path: The relative or absolute path to the log file (e.g., "logs/app.log").

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_json_format(self, enabled: bool) -> "Config":
        """
        Enables or disables JSON formatted output.

        When enabled, logs are emitted as structured JSON strings, which is ideal
        for ingestion by automated systems like ELK stack, Datadog or Splunk.

        Args:
            enabled: True to format logs as JSON, False for standard text.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_colored_output(self, enabled: bool) -> "Config":
        """
        Enables or disables ANSI colored output for console logs.

        Makes reading terminal logs easier by color-coding log levels (e.g., Error is red).

        Args:
            enabled: True to enable colors, False to disable them.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_profiling(self, enabled: bool) -> "Config":
        """
        Enables or disables performance profiling tracking.

        When enabled, you can use `logger.profile(...)` to measure block execution times.

        Args:
            enabled: True to enable profiling features.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_monitoring(self, enabled: bool) -> "Config":
        """
        Enables continuous background resource monitoring (CPU/Memory usage).

        Adds hardware telemetry metadata to your structured logs.

        Args:
            enabled: True to enable hardware monitoring.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_buffer_size(self, size: int) -> "Config":
        """
        Sets the internal capacity size for buffered logging.

        Args:
            size: The maximum number of log entries to buffer in memory before flushing.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_file_rotation(self, max_size: int, max_files: int) -> "Config":
        """
        Configures log file rotation based on file size.

        Args:
            max_size: Maximum size in bytes before rotating the file.
            max_files: Maximum number of backup files to retain.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_async(self, enabled: bool) -> "Config":
        """
        Enables asynchronous background logging.

        When enabled, logs are emitted in a background thread, heavily reducing
        I/O blocking in your application's critical path.

        Args:
            enabled: True to dispatch logs asynchronously.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_buffering(self, enabled: bool) -> "Config":
        """
        Enables buffered logging for increased throughput performance.

        Logs will be held in memory until the buffer is full or flushed,
        reducing the amount of disk/network write operations.

        Args:
            enabled: True to enable buffering.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_component_tracking(self, enabled: bool) -> "Config":
        """
        Enables tracking of hierarchical components and tasks.

        Allows the usage of `logger.track_component(...)` to build a localized
        tree of execution stages.

        Args:
            enabled: True to enable component tracking.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_chart_config(self, config: "VisualizationConfig") -> "Config":
        """
        Attaches a visualization configuration for automated chart plotting.

        Args:
            config: A fully formed `VisualizationConfig` object describing how
                    components/profiling should be charted.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_auto_generate_charts(self, enabled: bool) -> "Config":
        """
        Enables automatic chart generation.

        When the application finishes or logger is dropped, diagram files
        will be automatically written to the disk based on tracked components.

        Args:
            enabled: True to auto-generate charts.

        Returns:
            The modified Config instance for method chaining.
        """
        ...

    def with_chart_output_directory(self, path: str) -> "Config":
        """
        Sets the directory where auto-generated charts will be saved.

        Args:
            path: Relative or absolute directory path (e.g., "./diagrams").

        Returns:
            The modified Config instance for method chaining.
        """
        ...

class ContextManager:
    """
    An active context manager that adds attributes to logs within its block.

    Any log emitted within the `with` block will automatically inherit the contextual
    key-value pair provided when initializing this manager.
    """
    def __enter__(self) -> None: ...
    def __exit__(
        self,
        exc_type: Optional[Any],
        exc_value: Optional[Any],
        traceback: Optional[Any],
    ) -> bool: ...

class ProfileContext:
    """
    An active context manager that profiles the execution time of its block.

    Upon exiting the `with` block, a profiling log will be emitted containing
    the duration it took to execute the inner code footprint.
    """
    def __enter__(self) -> None: ...
    def __exit__(
        self,
        exc_type: Optional[Any],
        exc_value: Optional[Any],
        traceback: Optional[Any],
    ) -> bool: ...

class ComponentContext:
    """
    An active context manager that tracks log operations under a specific hierarchy.

    Useful for visualizing multi-step processes or generating gantt/timeline charts.
    """
    def __enter__(self) -> None: ...
    def __exit__(
        self,
        exc_type: Optional[Any],
        exc_value: Optional[Any],
        traceback: Optional[Any],
    ) -> bool: ...

class ComponentTrackerWrapper:
    """Provides read access to the components tracked during the application lifecycle."""

    def get_all_components(self) -> List[Dict[str, str]]:
        """
        Retrieves a list of dictionaries detailing all tracked components.

        Returns:
            A list where each dict contains keys like "id", "name", "status",
            "parent_id", and "duration_ms".
        """
        ...

    def count(self) -> int:
        """
        Retrieves the total number of components currently tracked.

        Returns:
            Total integer count.
        """
        ...

class VisualizationConfig:
    """
    Configuration builder for Mermaid.js charts and visualization generation.

    Use this to strictly control the design and layout of your profile charts and
    component hierarchy diagrams.
    """
    def __init__(self) -> None: ...
    def with_chart_type(self, chart_type: ChartType) -> "VisualizationConfig":
        """Sets the type of chart to generate ("flowchart", "timeline", "gantt")."""
        ...

    def with_direction(self, direction: Direction) -> "VisualizationConfig":
        """
        Sets the rendering direction of the chart.
        Applicable values include "TD" (Top-Down), "LR" (Left-Right), etc.
        """
        ...

    def set_timing(self, show_timing: bool) -> "VisualizationConfig":
        """If True, execution duration times are embedded directly into the chart nodes."""
        ...

    def set_memory(self, show_memory: bool) -> "VisualizationConfig":
        """If True, includes measured memory footprint sizes into chart nodes."""
        ...

    def set_metadata(self, show_metadata: bool) -> "VisualizationConfig":
        """If True, includes custom component metadata into the generated chart output."""
        ...

class Logger:
    """
    The main Telelog Logger instance for highly-performant structured logging.

    It supports structured key-value bindings, contextual persistence over scopes,
    performance profiling, component execution tracking, and visualization integrations.
    """

    def __init__(self, name: str) -> None:
        """
        Initializes a Logger with the default basic configuration.

        Args:
            name: The namespace or name of this logger (e.g., "my_app.network").
        """
        ...

    @staticmethod
    def with_config(name: str, config: Config) -> "Logger":
        """
        Creates a new Logger using a heavily customized Config object.

        Args:
            name: The namespace or name of this logger (e.g., "db_layer").
            config: A `telelog.Config` instance containing all output constraints.

        Returns:
            A robust Logger instance ready to log.
        """
        ...

    def get_config(self) -> Config:
        """Returns a clone of the current configuration bound to this logger."""
        ...

    def set_config(self, config: Config) -> None:
        """Updates or replaces the live configuration of the Logger."""
        ...

    def name(self) -> str:
        """Returns the namespace name of this Logger instance."""
        ...

    def debug(self, message: str) -> None:
        """
        Logs a standard message at the DEBUG level.
        Mostly used for extremely verbose diagnostic information.
        """
        ...

    def info(self, message: str) -> None:
        """
        Logs a standard message at the INFO level.
        Used for general, expected application lifecycle events.
        """
        ...

    def warning(self, message: str) -> None:
        """
        Logs a standard message at the WARNING level.
        Used for situations that are concerning but not fatal.
        """
        ...

    def error(self, message: str) -> None:
        """
        Logs a standard message at the ERROR level.
        Used for exceptions or operational failures.
        """
        ...

    def critical(self, message: str) -> None:
        """
        Logs a standard message at the CRITICAL level.
        Used immediately preceding severe application crashes or safety violations.
        """
        ...

    def debug_with(self, message: str, data: Sequence[Tuple[str, str]]) -> None:
        """
        Logs a message with bound structured data at the DEBUG level.

        Args:
            message: The string message to log.
            data: A sequence of key-value tuples (e.g., `[("user_id", "123")]`).
        """
        ...

    def info_with(self, message: str, data: Sequence[Tuple[str, str]]) -> None:
        """Logs a message with bound structured data at the INFO level."""
        ...

    def warning_with(self, message: str, data: Sequence[Tuple[str, str]]) -> None:
        """Logs a message with bound structured data at the WARNING level."""
        ...

    def error_with(self, message: str, data: Sequence[Tuple[str, str]]) -> None:
        """Logs a message with bound structured data at the ERROR level."""
        ...

    def critical_with(self, message: str, data: Sequence[Tuple[str, str]]) -> None:
        """Logs a message with bound structured data at the CRITICAL level."""
        ...

    def log_with(
        self, level: LogLevel, message: str, data: Sequence[Tuple[str, str]]
    ) -> None:
        """
        Dynamically logs a message with structured data using the specified log level.

        Args:
            level: Dynamic runtime representation of minimum level strings.
            message: The string message to log.
            data: A sequence of key-value tuples providing structured data.
        """
        ...

    def add_context(self, key: str, value: str) -> None:
        """
        Globally injects a persistent context key-value pair for this logger.
        Every log executed by this logger after this call will include this data.
        """
        ...

    def remove_context(self, key: str) -> None:
        """Removes an explicitly injected global context key."""
        ...

    def clear_context(self) -> None:
        """Clears all globally held context variables from the logger instance."""
        ...

    def with_context(self, key: str, value: str) -> ContextManager:
        """
        Retrieves a context manager that strictly scopes contextual data.

        Example:
            ```python
            with logger.with_context("request_id", "404"):
                logger.info("Verifying permissions")  # request_id=404 is included here

            logger.info("Finished") # request_id is gone here
            ```
        """
        ...

    def profile(self, operation: str) -> ProfileContext:
        """
        Retrieves a context manager to track execution time of nested code.

        Example:
            ```python
            with logger.profile("database_query"):
                db.execute("SELECT * FROM users")
            # Automatically logs: database_query Completed in 24ms
            ```
        """
        ...

    def track_component(self, name: str) -> ComponentContext:
        """
        Retrieves a context manager to track component hierarchies.
        Nested component trackings build a tree execution map for chart visualization.
        """
        ...

    def get_component_tracker(self) -> ComponentTrackerWrapper:
        """Returns the live tracker wrapper providing info on finished components."""
        ...

    def generate_visualization(
        self, chart_type: ChartType, output_path: Optional[str] = None
    ) -> str:
        """
        Generates a Mermaid.js markup text representing component flows.

        Args:
            chart_type: Format to render ("flowchart", "timeline", "gantt").
            output_path: File path to save output to (e.g., path/to/chart.mmd).

        Returns:
            The raw mermaid.js markup as a string.
        """
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

def create_logger(name: str) -> Logger:
    """
    Convenience factory to quickly yield a default, zero-configuration Logger.
    """
    ...
