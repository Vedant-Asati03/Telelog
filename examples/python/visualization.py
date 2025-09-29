#!/usr/bin/env python3
"""
Visualization Example
Demonstrates generating charts and visualizations from logged data.
"""
import time
import telelog as tl

def main():
    logger = tl.Logger("viz_demo")
    
    logger.info("Starting visualization demo")
    
    # Generate some activity to visualize
    with logger.track_component("api_gateway"):
        logger.info("Request received")
        
        with logger.profile("auth_check"):
            time.sleep(0.05)
            logger.info("Authentication verified")
        
        with logger.track_component("business_service"):
            with logger.profile("data_processing"):
                time.sleep(0.08)
                logger.info("Data processed")
    
    # Generate different types of visualizations
    print("\n📊 Generating visualizations...")
    
    flowchart = logger.generate_visualization("flowchart")
    print(f"✅ Flowchart generated ({len(flowchart)} chars)")
    print(f"📄 Flowchart content:\n{flowchart}\n")
    
    timeline = logger.generate_visualization("timeline")
    print(f"✅ Timeline generated ({len(timeline)} chars)")
    print(f"📄 Timeline content:\n{timeline}\n")
    
    gantt = logger.generate_visualization("gantt")
    print(f"✅ Gantt chart generated ({len(gantt)} chars)")
    print(f"📄 Gantt content:\n{gantt}\n")
    
    # Save charts to files
    import os
    os.makedirs("./viz_output", exist_ok=True)
    
    with open("./viz_output/flowchart.mmd", "w") as f:
        f.write(flowchart)
    print("💾 Flowchart saved to ./viz_output/flowchart.mmd")
    
    with open("./viz_output/timeline.mmd", "w") as f:
        f.write(timeline)
    print("💾 Timeline saved to ./viz_output/timeline.mmd")
    
    with open("./viz_output/gantt.mmd", "w") as f:
        f.write(gantt)
    print("💾 Gantt chart saved to ./viz_output/gantt.mmd")
    
    print("✅ Visualization example finished")
    print("💡 View the saved .mmd files or paste content into https://mermaid.live/")

if __name__ == "__main__":
    main()