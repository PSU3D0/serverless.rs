/*!
Self-documentation mechanism for serverless.rs.

This module provides utilities for generating and displaying
function metadata using the `--info` flag. This mechanism enables:

1. Automatic documentation of serverless functions
2. Resource recommendation for infrastructure-as-code tools
3. Platform compatibility validation
*/

use crate::requirements::Requirements;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP route information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteInfo {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Path pattern
    pub path: String,

    /// Optional description of what this route does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RouteInfo {
    /// Create a new route information
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            description: None,
        }
    }

    /// Add a description to the route
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Function metadata for self-documentation
///
/// This structure follows the JSON schema defined in the PRD [TECH-4]
/// and provides all the necessary information for the self-documentation mechanism.
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteInfo>,

    /// Additional metadata about the function
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl FunctionInfo {
    /// Create a new function information object
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            resources: Requirements::new(),
            routes: Vec::new(),
            metadata: HashMap::new(),
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

    /// Add custom metadata to the function
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Export the function information as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the function information for human-readable output
    pub fn format_for_display(&self) -> String {
        let mut output = format!("# Function: {}\n", self.name);

        if let Some(desc) = &self.description {
            output.push_str(&format!("\n## Description\n{}\n", desc));
        }

        // Format routes
        if !self.routes.is_empty() {
            output.push_str("\n## Routes\n");
            for route in &self.routes {
                output.push_str(&format!("- {} {}\n", route.method, route.path));
                if let Some(desc) = &route.description {
                    output.push_str(&format!("  Description: {}\n", desc));
                }
            }
        }

        // Format resource requirements
        output.push_str("\n## Resource Requirements\n");

        if !self.resources.required.is_empty() {
            output.push_str("\n### Required Resources\n");
            for (name, resource) in &self.resources.required {
                output.push_str(&format!("- {}: {}\n", name, resource.value));
                if let Some(desc) = &resource.description {
                    output.push_str(&format!("  Description: {}\n", desc));
                }
            }
        }

        if !self.resources.recommended.is_empty() {
            output.push_str("\n### Recommended Resources\n");
            for (name, resource) in &self.resources.recommended {
                output.push_str(&format!("- {}: {}\n", name, resource.value));
                if let Some(desc) = &resource.description {
                    output.push_str(&format!("  Description: {}\n", desc));
                }
            }
        }

        // Format platforms
        if !self.resources.platforms.is_empty() {
            output.push_str("\n## Supported Platforms\n");
            for platform in &self.resources.platforms {
                output.push_str(&format!("- {}\n", platform));
            }
        }

        // Format environment variables
        if !self.resources.environment.is_empty() {
            output.push_str("\n## Environment Variables\n");
            for env_var in &self.resources.environment {
                output.push_str(&format!("- {}\n", env_var));
            }
        }

        // Format metadata
        if !self.metadata.is_empty() {
            output.push_str("\n## Additional Metadata\n");
            for (key, value) in &self.metadata {
                output.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        output
    }
}

/// Display function information in the console
///
/// This function handles the output of function metadata in two formats:
/// 1. JSON format (when --json flag is present)
/// 2. Human-readable format (default)
pub fn display_info(info: &FunctionInfo) {
    // Check if JSON output is requested
    if check_json_flag() {
        if let Ok(json) = info.to_json() {
            println!("{}", json);
        } else {
            eprintln!("Error: Failed to serialize function information to JSON");
        }
    } else {
        // Display human-readable output
        println!("{}", info.format_for_display());
    }
}

/// Parse command-line arguments to check for the --info flag
///
/// Returns true if the --info flag is present, false otherwise.
pub fn check_info_flag() -> bool {
    std::env::args().any(|arg| arg == "--info")
}

/// Parse command-line arguments to check for the --json flag
///
/// Returns true if the --json flag is present, false otherwise.
/// This is used in conjunction with the --info flag to request JSON output.
fn check_json_flag() -> bool {
    std::env::args().any(|arg| arg == "--json")
}

/// Enum representing the requested output format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// JSON output format
    Json,
    /// Human-readable text output format
    Text,
}

/// Parse command-line arguments to determine the desired actions
///
/// This function provides more comprehensive argument parsing than the
/// individual flag check functions. It returns a tuple with:
/// 1. Whether the --info flag is present
/// 2. The requested output format (JSON or text)
pub fn parse_info_args() -> (bool, OutputFormat) {
    let info_requested = check_info_flag();
    let format = if check_json_flag() {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };

    (info_requested, format)
}

/// Display function information and exit if the --info flag is present
///
/// This is a convenience function that can be called at the start of the
/// main function to handle the --info flag automatically.
pub fn handle_info_request(info: &FunctionInfo) -> bool {
    let (info_requested, _) = parse_info_args();

    if info_requested {
        display_info(info);
        true
    } else {
        false
    }
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

    #[test]
    fn test_check_info_flag() {
        // Test when flag is not present
        assert!(!check_info_flag());

        // We can't modify env::args() directly, so we'll skip testing the positive case
        // This would require integration tests with actual command-line arguments
    }

    #[test]
    fn test_format_for_display() {
        let resources = Requirements::new()
            .recommend(Resource::new("memory", "256MB").with_description("Memory limit"))
            .require(Resource::new("cpu", "1x"))
            .platform("aws")
            .env_var("API_KEY");

        let info = FunctionInfo::new("test_function")
            .with_description("Test function description")
            .with_resources(resources)
            .add_route(RouteInfo::new("GET", "/test").with_description("Test endpoint"))
            .add_metadata("version", "1.0");

        let display = info.format_for_display();

        // Check that all important information is present in the display string
        assert!(display.contains("# Function: test_function"));
        assert!(display.contains("Test function description"));
        assert!(display.contains("GET /test"));
        assert!(display.contains("Test endpoint"));
        assert!(display.contains("Required Resources"));
        assert!(display.contains("cpu: 1x"));
        assert!(display.contains("Recommended Resources"));
        assert!(display.contains("memory: 256MB"));
        assert!(display.contains("Memory limit"));
        assert!(display.contains("Supported Platforms"));
        assert!(display.contains("aws"));
        assert!(display.contains("Environment Variables"));
        assert!(display.contains("API_KEY"));
        assert!(display.contains("Additional Metadata"));
        assert!(display.contains("version: 1.0"));
    }

    #[test]
    fn test_parse_info_args() {
        // Default case without arguments
        let (info_requested, format) = parse_info_args();
        assert!(!info_requested);
        assert!(matches!(format, OutputFormat::Text));

        // We can't modify env::args() directly, so we'll skip testing other cases
        // This would require integration tests with actual command-line arguments
    }
}
