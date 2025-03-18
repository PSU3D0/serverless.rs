/*!
Self-documentation mechanism for serverless.rs.

This module provides utilities for generating and displaying
function metadata using the --info flag.
*/

use crate::requirements::Requirements;
use serde::{Deserialize, Serialize};

/// HTTP route information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteInfo {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Path pattern
    pub path: String,
}

#[allow(dead_code)]
impl RouteInfo {
    /// Create a new route information
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
        }
    }
}

/// Function metadata for self-documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,

    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Resource requirements and recommendations
    pub resources: Requirements,

    /// HTTP routes exposed by the function
    #[serde(default)]
    pub routes: Vec<RouteInfo>,
}

#[allow(dead_code)]
impl FunctionInfo {
    /// Create a new function information
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            resources: Requirements::new(),
            routes: Vec::new(),
        }
    }

    /// Add a description to the function
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the resource requirements
    pub fn with_resources(mut self, resources: Requirements) -> Self {
        self.resources = resources;
        self
    }

    /// Add an HTTP route
    pub fn add_route(mut self, route: RouteInfo) -> Self {
        self.routes.push(route);
        self
    }

    /// Export the function information as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Display function information in the console
#[allow(dead_code)]
pub fn display_info(info: &FunctionInfo) {
    if let Ok(json) = info.to_json() {
        println!("{}", json);
    } else {
        println!("Error: Failed to serialize function information");
    }
}

/// Parse command-line arguments to check for the --info flag
///
/// Returns true if the --info flag is present, false otherwise.
#[allow(dead_code)]
pub fn check_info_flag() -> bool {
    std::env::args().any(|arg| arg == "--info")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirements::Resource;

    #[test]
    fn test_function_info() {
        let resources = Requirements::new()
            .recommend(Resource::new("memory", "128MB"))
            .require(Resource::new("cpu", "1x"))
            .platform("aws")
            .platform("cloudflare")
            .env_var("API_KEY");

        let info = FunctionInfo::new("api_handler")
            .with_description("API endpoint for user data")
            .with_resources(resources)
            .add_route(RouteInfo::new("GET", "/users"))
            .add_route(RouteInfo::new("POST", "/users"));

        assert_eq!(info.name, "api_handler");
        assert_eq!(
            info.description,
            Some("API endpoint for user data".to_string())
        );
        assert_eq!(info.resources.platforms.len(), 2);
        assert_eq!(info.routes.len(), 2);

        let json = info.to_json().unwrap();
        assert!(json.contains("api_handler"));
        assert!(json.contains("/users"));
    }
}
