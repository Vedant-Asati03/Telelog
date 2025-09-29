#!/usr/bin/env python3
"""
Context Management Example
Demonstrates adding, managing, and clearing logging contexts.
"""
import telelog as tl

def main():
    logger = tl.Logger("context_demo")
    
    # Basic logging without context
    logger.info("Starting application")
    
    # Add request context
    logger.add_context("request_id", "req_123")
    logger.add_context("user_id", "user_456")
    logger.info("Processing request")  # Will include context
    
    # Add more context
    logger.add_context("session_id", "sess_789")
    logger.info("User authenticated")  # Will include all context
    
    # Remove specific context
    logger.remove_context("session_id")
    logger.info("Session context removed")  # session_id gone
    
    # Clear all context
    logger.clear_context()
    logger.info("All context cleared")  # No context
    
    print("✅ Context management example finished")

if __name__ == "__main__":
    main()