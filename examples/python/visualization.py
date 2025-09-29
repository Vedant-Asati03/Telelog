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
    
    timeline = logger.generate_visualization("timeline")
    print(f"✅ Timeline generated ({len(timeline)} chars)")
    
    gantt = logger.generate_visualization("gantt")
    print(f"✅ Gantt chart generated ({len(gantt)} chars)")
    
    print("✅ Visualization example finished")
    print("💡 Paste the generated content into https://mermaid.live/ to view")

if __name__ == "__main__":
    main()