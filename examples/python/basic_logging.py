#!/usr/bin/env python3
"""
Basic Logging Example
Demonstrates core logging functionality with different log levels.
"""
import telelog as tl

def main():
    # Create logger
    logger = tl.Logger("basic_demo")
    
    # Different log levels
    logger.debug("Debug message")
    logger.info("Application started")
    logger.warning("This is a warning")
    logger.error("This is an error")
    logger.critical("This is critical")
    
    # Structured logging with context
    logger.add_context("user_id", "12345")
    logger.add_context("action", "login") 
    logger.add_context("ip", "192.168.1.1")
    logger.info("User action")
    logger.clear_context()
    
    logger.info("Basic logging demo complete")
    print("✅ Basic logging example finished")

if __name__ == "__main__":
    main()