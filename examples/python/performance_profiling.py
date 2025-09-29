#!/usr/bin/env python3
"""
Performance Profiling Example
Demonstrates timing operations and performance measurement.
"""
import time
import telelog as tl

def main():
    logger = tl.Logger("perf_demo")
    
    logger.info("Starting performance demo")
    
    # Simple profiling with context manager
    with logger.profile("database_query"):
        time.sleep(0.1)  # Simulate database work
        logger.info("Query executed")
    
    # Nested profiling
    with logger.profile("request_processing"):
        logger.info("Processing request")
        
        with logger.profile("validation"):
            time.sleep(0.05)  # Simulate validation
            logger.info("Input validated")
        
        with logger.profile("business_logic"):
            time.sleep(0.08)  # Simulate processing
            logger.info("Business logic executed")
        
        logger.info("Request completed")
    
    print("✅ Performance profiling example finished")

if __name__ == "__main__":
    main()