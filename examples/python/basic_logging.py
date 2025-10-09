#!/usr/bin/env python3
"""Basic logging with different log levels and structured data."""

import telelog as tl

def main():
    logger = tl.Logger("basic_demo")
    
    logger.debug("Debug message")
    logger.info("Application started")
    logger.warning("This is a warning")
    logger.error("This is an error")
    logger.critical("This is critical")
    
    logger.info_with("User action", [
        ("user_id", "12345"),
        ("action", "login"),
        ("ip", "192.168.1.1")
    ])
    
    logger.warning_with("Resource usage", [
        ("cpu", "85%"),
        ("memory", "78%")
    ])
    
    logger.info("Basic logging demo complete")
    print("✅ Basic logging example finished")

if __name__ == "__main__":
    main()