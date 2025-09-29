//! Visualization Example
//! Demonstrates generating charts and visualizations from logged data.

use std::{thread, time::Duration};
use telelog::Logger;

fn main() {
    let logger = Logger::new("viz_demo");

    logger.info("Starting visualization demo");

    // Generate some activity to visualize
    {
        let _api_gateway = logger.track_component("api_gateway");
        logger.info("Request received");

        {
            let _auth_timer = logger.profile("auth_check");
            thread::sleep(Duration::from_millis(50));
            logger.info("Authentication verified");
        }

        {
            let _business_service = logger.track_component("business_service");
            let _data_timer = logger.profile("data_processing");
            thread::sleep(Duration::from_millis(80));
            logger.info("Data processed");
        }
    }

    // Generate different types of visualizations using MermaidGenerator
    println!("\n📊 Generating visualizations...");

    use telelog::{ChartConfig, ChartType, MermaidGenerator};

    let tracker = logger.get_component_tracker();

    // Generate flowchart
    let flowchart_config = ChartConfig::new().with_chart_type(ChartType::Flowchart);
    let flowchart_generator = MermaidGenerator::new(flowchart_config);
    let flowchart = flowchart_generator
        .generate_diagram(tracker)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Flowchart generated ({} chars)", flowchart.len());
    
    // Generate timeline
    let timeline_config = ChartConfig::new().with_chart_type(ChartType::Timeline);
    let timeline_generator = MermaidGenerator::new(timeline_config);
    let timeline = timeline_generator
        .generate_diagram(tracker)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Timeline generated ({} chars)", timeline.len());
    
    // Generate Gantt chart
    let gantt_config = ChartConfig::new().with_chart_type(ChartType::Gantt);
    let gantt_generator = MermaidGenerator::new(gantt_config);
    let gantt = gantt_generator
        .generate_diagram(tracker)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Gantt chart generated ({} chars)", gantt.len());

    // Display the generated charts
    println!("\n📊 Generated Charts:");
    println!("\n🔸 FLOWCHART:");
    println!("{}", flowchart);
    
    println!("\n🔸 TIMELINE:");
    println!("{}", timeline);
    
    println!("\n🔸 GANTT CHART:");
    println!("{}", gantt);

    // Save to files
    use std::fs;
    fs::create_dir_all("./viz_output").ok();
    
    fs::write("./viz_output/flowchart.mmd", &flowchart).ok();
    fs::write("./viz_output/timeline.mmd", &timeline).ok();
    fs::write("./viz_output/gantt.mmd", &gantt).ok();

    println!("\n✅ Visualization example finished");
    println!("� Charts saved to ./viz_output/ directory");
    println!("💡 View at: https://mermaid.live/ or in VS Code with Mermaid extension");
}
