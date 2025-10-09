#!/usr/bin/env python3
"""Genera    print(f"✅ Gantt chart generated ({len(gantt)} chars)")
    
    print(f"\n🔸 FLOWCHART:\n{flowchart}\n") charts and visualizations from logged data."""

import time
import telelog as tl

def main():
    logger = tl.Logger("viz_demo")
    
    logger.info("Starting visualization demo")
    
    with logger.track_component("api_gateway"):
        logger.info("Request received")
        
        with logger.profile("auth_check"):
            time.sleep(0.05)
            logger.info("Authentication verified")
        
        with logger.track_component("business_service"):
            with logger.profile("data_processing"):
                time.sleep(0.08)
                logger.info("Data processed")
    
    print("\n📊 Generating visualizations...")
    
    flowchart = logger.generate_visualization("flowchart")
    print(f"✅ Flowchart generated ({len(flowchart)} chars)")
    
    timeline = logger.generate_visualization("timeline")
    print(f"✅ Timeline generated ({len(timeline)} chars)")
    
    gantt = logger.generate_visualization("gantt")
    print(f"✅ Gantt chart generated ({len(gantt)} chars)")
    
    # Display charts
    print(f"\n� FLOWCHART:\n{flowchart}\n")
    print(f"🔸 TIMELINE:\n{timeline}\n")
    print(f"🔸 GANTT CHART:\n{gantt}\n")
    
    import os
    os.makedirs("./viz_output", exist_ok=True)
    
    with open("./viz_output/flowchart.mmd", "w") as f:
        f.write(flowchart)
    with open("./viz_output/timeline.mmd", "w") as f:
        f.write(timeline)
    with open("./viz_output/gantt.mmd", "w") as f:
        f.write(gantt)
    
    print("✅ Visualization example finished")
    print("� Charts saved to ./viz_output/ directory")
    print("💡 View at: https://mermaid.live/")

if __name__ == "__main__":
    main()