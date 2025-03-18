/*!
Platform-agnostic context abstraction for serverless.rs.

This module provides a unified context abstraction that works across
different serverless platforms.
*/

use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// A platform-agnostic execution context for serverless functions
#[derive(Debug, Clone)]
pub struct Context {
    /// Unique request ID
    request_id: String,

    /// Function name
    function_name: String,

    /// Function version/alias
    function_version: String,

    /// Memory limit in MB
    memory_limit: Option<u32>,

    /// Remaining execution time
    remaining_time: Option<Duration>,

    /// Function execution deadline
    deadline: Option<SystemTime>,

    /// Environment variables
    env_vars: HashMap<String, String>,

    /// Platform-specific context data
    platform_data: Value,
}

impl Context {
    /// Creates a new empty context
    pub fn new() -> Self {
        Self {
            request_id: String::new(),
            function_name: String::new(),
            function_version: String::new(),
            memory_limit: None,
            remaining_time: None,
            deadline: None,
            env_vars: HashMap::new(),
            platform_data: Value::Null,
        }
    }

    /// Returns the unique request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Sets the request ID for this context
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = request_id.into();
        self
    }

    /// Returns the function name
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Sets the function name for this context
    pub fn with_function_name(mut self, function_name: impl Into<String>) -> Self {
        self.function_name = function_name.into();
        self
    }

    /// Returns the function version/alias
    pub fn function_version(&self) -> &str {
        &self.function_version
    }

    /// Sets the function version for this context
    pub fn with_function_version(mut self, function_version: impl Into<String>) -> Self {
        self.function_version = function_version.into();
        self
    }

    /// Returns the memory limit in MB, if available
    pub fn memory_limit(&self) -> Option<u32> {
        self.memory_limit
    }

    /// Sets the memory limit for this context
    pub fn with_memory_limit(mut self, memory_limit: u32) -> Self {
        self.memory_limit = Some(memory_limit);
        self
    }

    /// Returns the remaining execution time, if available
    pub fn remaining_time(&self) -> Option<Duration> {
        self.remaining_time
    }

    /// Sets the remaining execution time for this context
    pub fn with_remaining_time(mut self, remaining_time: Duration) -> Self {
        self.remaining_time = Some(remaining_time);
        self
    }

    /// Returns the function execution deadline, if available
    pub fn deadline(&self) -> Option<SystemTime> {
        self.deadline
    }

    /// Sets the execution deadline for this context
    pub fn with_deadline(mut self, deadline: SystemTime) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Returns the environment variables
    pub fn env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Sets an environment variable for this context
    pub fn with_env_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(name.into(), value.into());
        self
    }

    /// Returns an environment variable by name
    pub fn env_var(&self, name: &str) -> Option<&String> {
        self.env_vars.get(name)
    }

    /// Returns the platform-specific context data
    pub fn platform_data(&self) -> &Value {
        &self.platform_data
    }

    /// Sets the platform-specific data for this context
    pub fn with_platform_data(mut self, platform_data: Value) -> Self {
        self.platform_data = platform_data;
        self
    }

    /// Gets a typed value from platform-specific data
    pub fn get_platform_data<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Option<T> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.platform_data;

        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(value) = obj.get(part) {
                    current = value;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        serde_json::from_value(current.clone()).ok()
    }

    /// Log a message to the platform-specific logging system
    /// This is a minimal implementation that will be enhanced by platform adapters
    pub fn log(&self, level: &str, message: &str) {
        println!("[{}] {} - {}", level, self.request_id, message);
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_builder() {
        let ctx = Context::new()
            .with_request_id("req-123")
            .with_function_name("test-function")
            .with_function_version("1.0")
            .with_memory_limit(128)
            .with_env_var("DATABASE_URL", "postgres://localhost/test");

        assert_eq!(ctx.request_id(), "req-123");
        assert_eq!(ctx.function_name(), "test-function");
        assert_eq!(ctx.function_version(), "1.0");
        assert_eq!(ctx.memory_limit(), Some(128));
        assert_eq!(
            ctx.env_var("DATABASE_URL"),
            Some(&"postgres://localhost/test".to_string())
        );
    }

    #[test]
    fn test_platform_data() {
        let platform_data = json!({
            "aws": {
                "region": "us-east-1",
                "account_id": "123456789012",
                "function": {
                    "arn": "arn:aws:lambda:us-east-1:123456789012:function:test"
                }
            }
        });

        let ctx = Context::new().with_platform_data(platform_data);

        let region: String = ctx.get_platform_data("aws.region").unwrap();
        assert_eq!(region, "us-east-1");

        let arn: String = ctx.get_platform_data("aws.function.arn").unwrap();
        assert_eq!(arn, "arn:aws:lambda:us-east-1:123456789012:function:test");

        let unknown: Option<String> = ctx.get_platform_data("aws.unknown");
        assert!(unknown.is_none());
    }
}
