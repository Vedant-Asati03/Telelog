# Telelog Examples

This directory contains **matching examples** for both Rust and Python, demonstrating all telelog features in a clear, organized manner.

## 📁 Structure

```
examples/
├── 01_basic_logging.rs      # Rust examples (numbered for clarity)
├── 02_context_management.rs
├── 03_performance_profiling.rs
├── 04_component_tracking.rs
├── 05_visualization.rs
├── 06_async_logging.rs
└── python/                  # Python examples
    ├── basic_logging.py
    ├── context_management.py
    ├── performance_profiling.py
    ├── component_tracking.py
    ├── visualization.py
    └── async_logging.py
```

## 🦀 Rust Examples

### Running Rust Examples
```bash
cargo run --example 01_basic_logging
cargo run --example 02_context_management
cargo run --example 03_performance_profiling
cargo run --example 04_component_tracking
cargo run --example 05_visualization
cargo run --example 06_async_logging
```

## 🐍 Python Examples

### Setup
```bash
# Install telelog in development mode
maturin develop --release
```

### Running Python Examples
```bash
python examples/python/basic_logging.py
python examples/python/context_management.py
python examples/python/performance_profiling.py
python examples/python/component_tracking.py
python examples/python/visualization.py
python examples/python/async_logging.py
```

## 📋 Feature Coverage

| Feature                   | Rust Example                  | Python Example             | Description                       |
| ------------------------- | ----------------------------- | -------------------------- | --------------------------------- |
| **Basic Logging**         | `01_basic_logging.rs`         | `basic_logging.py`         | Log levels, structured logging    |
| **Context Management**    | `02_context_management.rs`    | `context_management.py`    | Add/remove/clear context          |
| **Performance Profiling** | `03_performance_profiling.rs` | `performance_profiling.py` | Time operations, nested profiling |
| **Component Tracking**    | `04_component_tracking.rs`    | `component_tracking.py`    | Track architectural components    |
| **Visualization**         | `05_visualization.rs`         | `visualization.py`         | Generate charts and diagrams      |
| **Async/Concurrency**     | `06_async_logging.rs`         | `async_logging.py`         | Multi-threaded/async patterns     |

## 🔧 Key Differences

- **Rust**: Uses RAII with guards/timers that automatically log when dropped
- **Python**: Uses context managers (`with` statements) for scoped operations
- **Both**: Provide identical functionality with language-appropriate patterns

## 💡 Tips

- Each example is **self-contained** and can be run independently
- Copy patterns from examples that match your use case
- Use visualization examples to debug performance issues