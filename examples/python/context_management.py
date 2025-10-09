#!/usr/bin/env python3
"""Managing persistent context that applies to multiple log entries."""

import telelog as tl

def main():
    logger = tl.Logger("context_demo")
    
    logger.info("Starting application")
    
    logger.add_context("request_id", "req_123")
    logger.add_context("user_id", "user_456")
    logger.info("Processing request")
    
    logger.add_context("session_id", "sess_789")
    logger.info("User authenticated")
    
    logger.remove_context("session_id")
    logger.info("Session context removed")
    
    logger.clear_context()
    logger.info("All context cleared")
    
    with logger.with_context("temp_id", "tmp_999"):
        logger.info("Inside scoped context")
    
    logger.info("Outside scoped context")
    
    print("✅ Context management example finished")

if __name__ == "__main__":
    main()