#!/usr/bin/env python3
"""
Component Tracking Example
Demonstrates tracking architectural components and their relationships.
"""
import time
import telelog as tl

def main():
    logger = tl.Logger("component_demo")
    
    logger.info("Starting component tracking demo")
    
    # Track web server component
    with logger.track_component("web_server"):
        logger.info("Web server handling request")
        
        # Track authentication service
        with logger.track_component("auth_service"):
            time.sleep(0.05)  # Simulate auth work
            logger.info("User authenticated")
        
        # Track database layer
        with logger.track_component("database"):
            time.sleep(0.08)  # Simulate DB work
            logger.info("Data retrieved")
        
        # Track response generation
        with logger.track_component("response_builder"):
            time.sleep(0.03)  # Simulate response building
            logger.info("Response generated")
        
        logger.info("Request completed")
    
    print("✅ Component tracking example finished")

if __name__ == "__main__":
    main()