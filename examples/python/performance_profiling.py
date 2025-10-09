#!/usr/bin/env python3
"""Tracking code execution time with performance profiling."""

import time
import telelog as tl

def main():
    logger = tl.Logger("perf_demo")
    
    logger.info("Starting performance demo")
    
    with logger.profile("database_query"):
        time.sleep(0.1)
        logger.info("Query executed")
    
    with logger.profile("request_processing"):
        logger.info("Processing request")
        
        with logger.profile("validation"):
            time.sleep(0.05)
            logger.info("Input validated")
        
        with logger.profile("business_logic"):
            time.sleep(0.08)
            logger.info("Business logic executed")
        
        logger.info("Request completed")
    
    print("✅ Performance profiling example finished")

if __name__ == "__main__":
    main()