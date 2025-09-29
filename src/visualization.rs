//! Mermaid visualization for component tracking

use crate::component::{Component, ComponentStatus, ComponentTracker};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Chart configuration for Mermaid visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart type (flowchart, timeline, etc.)
    pub chart_type: ChartType,
    /// Direction for flowcharts
    pub direction: Direction,
    /// Include timing information
    pub show_timing: bool,
    /// Include memory information
    pub show_memory: bool,
    /// Include metadata in nodes
    pub show_metadata: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChartType {
    /// Flowchart showing component dependencies
    Flowchart,
    /// Timeline showing component execution order
    Timeline,
    /// Gantt chart showing component durations
    Gantt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    /// Top to bottom
    TopDown,
    /// Bottom to top
    BottomUp,
    /// Left to right
    LeftRight,
    /// Right to left
    RightLeft,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Flowchart,
            direction: Direction::TopDown,
            show_timing: true,
            show_memory: false,
            show_metadata: false,
        }
    }
}

impl ChartConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_timing(mut self, show_timing: bool) -> Self {
        self.show_timing = show_timing;
        self
    }

    pub fn with_memory(mut self, show_memory: bool) -> Self {
        self.show_memory = show_memory;
        self
    }

    pub fn with_metadata(mut self, show_metadata: bool) -> Self {
        self.show_metadata = show_metadata;
        self
    }
}

impl Direction {
    fn to_mermaid(&self) -> &'static str {
        match self {
            Direction::TopDown => "TB",
            Direction::BottomUp => "BT",
            Direction::LeftRight => "LR",
            Direction::RightLeft => "RL",
        }
    }
}

/// Mermaid visualization generator
pub struct MermaidGenerator {
    config: ChartConfig,
}

impl MermaidGenerator {
    pub fn new(config: ChartConfig) -> Self {
        Self { config }
    }

    /// Generate Mermaid diagram from component tracker
    pub fn generate_diagram(&self, tracker: &ComponentTracker) -> Result<String, String> {
        let components = tracker.get_components();

        match self.config.chart_type {
            ChartType::Flowchart => self.generate_flowchart(&components),
            ChartType::Timeline => self.generate_timeline(&components),
            ChartType::Gantt => self.generate_gantt(&components),
        }
    }

    /// Generate flowchart diagram
    fn generate_flowchart(
        &self,
        components: &HashMap<String, Component>,
    ) -> Result<String, String> {
        let mut diagram = String::new();

        // Chart declaration with direction
        writeln!(diagram, "flowchart {}", self.config.direction.to_mermaid())
            .map_err(|e| format!("Failed to write diagram: {}", e))?;

        // Add nodes
        for (id, component) in components {
            let node_id = self.sanitize_id(id);
            let node_label = self.format_node_label(component);
            let node_style = self.get_node_style(&component.status);

            writeln!(diagram, "    {}{}", node_id, node_label)
                .map_err(|e| format!("Failed to write node: {}", e))?;

            if !node_style.is_empty() {
                writeln!(diagram, "    class {} {}", node_id, node_style)
                    .map_err(|e| format!("Failed to write node style: {}", e))?;
            }
        }

        // Add edges (parent-child relationships)
        for (id, component) in components {
            for child_id in &component.children {
                writeln!(
                    diagram,
                    "    {} --> {}",
                    self.sanitize_id(id),
                    self.sanitize_id(child_id)
                )
                .map_err(|e| format!("Failed to write edge: {}", e))?;
            }
        }

        // Add CSS classes for styling
        writeln!(
            diagram,
            "\n    classDef success fill:#d4edda,stroke:#28a745"
        )
        .map_err(|e| format!("Failed to write CSS: {}", e))?;
        writeln!(diagram, "    classDef failed fill:#f8d7da,stroke:#dc3545")
            .map_err(|e| format!("Failed to write CSS: {}", e))?;
        writeln!(diagram, "    classDef running fill:#fff3cd,stroke:#ffc107")
            .map_err(|e| format!("Failed to write CSS: {}", e))?;
        writeln!(
            diagram,
            "    classDef cancelled fill:#e2e3e5,stroke:#6c757d"
        )
        .map_err(|e| format!("Failed to write CSS: {}", e))?;

        Ok(diagram)
    }

    /// Generate timeline diagram
    fn generate_timeline(&self, components: &HashMap<String, Component>) -> Result<String, String> {
        let mut diagram = String::new();

        writeln!(diagram, "timeline").map_err(|e| format!("Failed to write timeline: {}", e))?;
        writeln!(diagram, "    title Component Execution Timeline")
            .map_err(|e| format!("Failed to write title: {}", e))?;

        // Sort components by start time (newest first for timeline)
        let mut sorted_components: Vec<_> = components.values().collect();
        sorted_components.sort_by_key(|c| std::cmp::Reverse(c.start_time));

        // Group components by sections for timeline
        for component in sorted_components {
            let label = if self.config.show_timing {
                if let Some(duration) = component.duration() {
                    format!(
                        "{} : {:.2}ms",
                        component.name,
                        duration.as_secs_f64() * 1000.0
                    )
                } else {
                    format!("{} : running", component.name)
                }
            } else {
                component.name.clone()
            };

            writeln!(diagram, "    {}", label)
                .map_err(|e| format!("Failed to write timeline entry: {}", e))?;
        }

        Ok(diagram)
    }

    /// Generate Gantt chart
    fn generate_gantt(&self, components: &HashMap<String, Component>) -> Result<String, String> {
        let mut diagram = String::new();

        writeln!(diagram, "gantt").map_err(|e| format!("Failed to write gantt: {}", e))?;
        writeln!(diagram, "    title Component Execution Gantt Chart")
            .map_err(|e| format!("Failed to write title: {}", e))?;
        writeln!(diagram, "    dateFormat x")
            .map_err(|e| format!("Failed to write dateFormat: {}", e))?;
        writeln!(diagram, "    axisFormat %L")
            .map_err(|e| format!("Failed to write axisFormat: {}", e))?;

        // Find earliest start time as baseline
        let baseline = components
            .values()
            .map(|c| c.start_time)
            .min()
            .unwrap_or_else(std::time::Instant::now);

        for component in components.values() {
            let start_ms = component.start_time.duration_since(baseline).as_millis();
            let end_ms = if let Some(end_time) = component.end_time {
                end_time.duration_since(baseline).as_millis()
            } else {
                std::time::Instant::now()
                    .duration_since(baseline)
                    .as_millis()
            };

            let status_marker = match component.status {
                ComponentStatus::Success => "",
                ComponentStatus::Failed(_) => " :crit",
                ComponentStatus::Running => " :active",
                ComponentStatus::Cancelled => " :done",
            };

            writeln!(
                diagram,
                "    {} {}{} : {}, {}",
                component.name.replace(' ', "_"),
                component.name,
                status_marker,
                start_ms,
                end_ms
            )
            .map_err(|e| format!("Failed to write gantt entry: {}", e))?;
        }

        Ok(diagram)
    }

    /// Format node label with optional timing and metadata
    fn format_node_label(&self, component: &Component) -> String {
        let mut parts = vec![component.name.clone()];

        if self.config.show_timing {
            if let Some(duration) = component.duration() {
                parts.push(format!("{:.1}ms", duration.as_secs_f64() * 1000.0));
            } else {
                parts.push("running".to_string());
            }
        }

        if self.config.show_memory {
            if let Some(memory) = component.metadata.memory_bytes {
                parts.push(format!("{}B", memory));
            }
        }

        if self.config.show_metadata && !component.metadata.custom.is_empty() {
            for (key, value) in &component.metadata.custom {
                parts.push(format!("{}:{}", key, value));
            }
        }

        // Join with <br/> tags for HTML-style line breaks in Mermaid
        format!("[\"{}\"]", parts.join("<br/>"))
    }

    /// Get CSS class for component status
    fn get_node_style(&self, status: &ComponentStatus) -> &'static str {
        match status {
            ComponentStatus::Success => "success",
            ComponentStatus::Failed(_) => "failed",
            ComponentStatus::Running => "running",
            ComponentStatus::Cancelled => "cancelled",
        }
    }

    /// Sanitize ID for use in Mermaid
    fn sanitize_id(&self, id: &str) -> String {
        // Replace invalid characters and ensure ID doesn't start with number
        let cleaned = id.replace(['-', ' ', '.'], "_");
        if cleaned.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            format!("node_{}", cleaned)
        } else {
            cleaned
        }
    }

    /// Generate diagram and save to .mmd file
    pub fn save_mmd(&self, tracker: &ComponentTracker, output_path: &Path) -> Result<(), String> {
        let diagram = self.generate_diagram(tracker)?;
        let mmd_path = output_path.with_extension("mmd");
        fs::write(&mmd_path, diagram)
            .map_err(|e| format!("Failed to write mermaid file: {}", e))?;
        Ok(())
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new(ChartConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{ComponentStatus, ComponentTracker};

    #[test]
    fn test_flowchart_generation() {
        let tracker = ComponentTracker::new();
        let parent_id = tracker.start_component("Parent");
        let child_id = tracker.start_component("Child");

        let _ = tracker.end_component(&child_id, ComponentStatus::Success);
        let _ = tracker.end_component(&parent_id, ComponentStatus::Success);

        let generator = MermaidGenerator::default();
        let diagram = generator.generate_diagram(&tracker).unwrap();

        assert!(diagram.contains("flowchart TB"));
        assert!(diagram.contains("Parent"));
        assert!(diagram.contains("Child"));
        assert!(diagram.contains("-->"));
    }

    #[test]
    fn test_chart_config() {
        let config = ChartConfig::new()
            .with_chart_type(ChartType::Timeline)
            .with_direction(Direction::LeftRight)
            .with_timing(true);

        assert_eq!(config.chart_type, ChartType::Timeline);
        assert_eq!(config.direction, Direction::LeftRight);
        assert!(config.show_timing);
    }

    #[test]
    fn test_node_label_formatting() {
        let generator =
            MermaidGenerator::new(ChartConfig::new().with_timing(true).with_memory(true));

        let tracker = ComponentTracker::new();
        let id = tracker.start_component("Test");

        // Add some metadata
        let mut metadata = crate::component::ComponentMetadata::new();
        metadata.memory_bytes = Some(1024);
        tracker.update_metadata(&id, metadata).unwrap();

        let _ = tracker.end_component(&id, ComponentStatus::Success);

        let components = tracker.get_components();
        let component = components.values().next().unwrap();

        let label = generator.format_node_label(component);
        assert!(label.contains("Test"));
        assert!(label.contains("1024B"));
    }
}
