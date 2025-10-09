#!/usr/bin/env python3
"""Tracking different components of your application."""

import time
import telelog as tl

def main():
    logger = tl.Logger("component_demo")
    
    logger.info("Starting component tracking demo")
    
    with logger.track_component("web_server"):
        logger.info("Web server handling request")
        
        with logger.track_component("auth_service"):
            time.sleep(0.05)
            logger.info("User authenticated")
        
        with logger.track_component("database"):
            time.sleep(0.08)
            logger.info("Data retrieved")
        
        with logger.track_component("response_builder"):
            time.sleep(0.03)
            logger.info("Response generated")
        
        logger.info("Request completed")
    
    print("✅ Component tracking example finished")

if __name__ == "__main__":
    main()