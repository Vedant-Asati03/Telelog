//! Generating charts and visualizations from logged data.

use std::{thread, time::Duration};
use telelog::Logger;

fn main() {
    let logger = Logger::new("viz_demo");

    logger.info("Starting visualization demo");

    {
        let _api_gateway = logger.track_component("api_gateway");
        logger.info_with("Request received", &[("endpoint", "/api/data")]);

        {
            let _auth_timer = logger.profile("auth_check");
            thread::sleep(Duration::from_millis(50));
            logger.info("Authentication verified");
        }

        {
            let _business_service = logger.track_component("business_service");
            let _data_timer = logger.profile("data_processing");
            thread::sleep(Duration::from_millis(80));
            logger.info_with("Data processed", &[("records", "150")]);
        }
    }

    println!("\n📊 Generating visualizations...");

    use telelog::ChartType;

    let flowchart = logger
        .generate_visualization(ChartType::Flowchart, None)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Flowchart generated ({} chars)", flowchart.len());

    let timeline = logger
        .generate_visualization(ChartType::Timeline, None)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Timeline generated ({} chars)", timeline.len());

    let gantt = logger
        .generate_visualization(ChartType::Gantt, None)
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("✅ Gantt chart generated ({} chars)", gantt.len());

    println!("\n📊 Generated Charts:");
    println!("\n🔸 FLOWCHART:");
    println!("{}", flowchart);

    println!("\n🔸 TIMELINE:");
    println!("{}", timeline);

    println!("\n🔸 GANTT CHART:");
    println!("{}", gantt);

    use std::fs;
    fs::create_dir_all("./viz_output").ok();

    fs::write("./viz_output/flowchart.mmd", &flowchart).ok();
    fs::write("./viz_output/timeline.mmd", &timeline).ok();
    fs::write("./viz_output/gantt.mmd", &gantt).ok();

    println!("\n✅ Visualization example finished");
    println!("💾 Charts saved to ./viz_output/ directory");
    println!("💡 View at: https://mermaid.live/ or in VS Code with Mermaid extension");
}
