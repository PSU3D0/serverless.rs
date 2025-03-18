/*!
Resource requirement annotations for serverless.rs.

This module provides types and utilities for defining and accessing
resource requirements for serverless functions.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource specification for serverless functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Name of the resource (e.g., "memory", "timeout", "concurrency")
    pub name: String,

    /// Value of the resource with units if applicable (e.g., "128MB", "30s", "10")
    pub value: String,

    /// Description of the resource for documentation purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Resource {
    /// Create a new resource specification
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            description: None,
        }
    }

    /// Add a description to the resource
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Requirements specification for serverless functions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requirements {
    /// Recommended resources for optimal function performance
    #[serde(default)]
    pub recommended: HashMap<String, Resource>,

    /// Required resources that must be provided for the function to work
    #[serde(default)]
    pub required: HashMap<String, Resource>,

    /// Supported platforms for this function
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Environment variables used by this function
    #[serde(default)]
    pub environment: Vec<String>,
}

impl Requirements {
    /// Create a new empty requirements specification
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a recommended resource
    pub fn recommend(mut self, resource: Resource) -> Self {
        self.recommended.insert(resource.name.clone(), resource);
        self
    }

    /// Add a required resource
    pub fn require(mut self, resource: Resource) -> Self {
        self.required.insert(resource.name.clone(), resource);
        self
    }

    /// Add a supported platform
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platforms.push(platform.into());
        self
    }

    /// Add an environment variable
    pub fn env_var(mut self, name: impl Into<String>) -> Self {
        self.environment.push(name.into());
        self
    }

    /// Get a recommended resource by name
    pub fn get_recommended(&self, name: &str) -> Option<&Resource> {
        self.recommended.get(name)
    }

    /// Get a required resource by name
    pub fn get_required(&self, name: &str) -> Option<&Resource> {
        self.required.get(name)
    }

    /// Check if a platform is supported
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.contains(&platform.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource =
            Resource::new("memory", "128MB").with_description("Memory limit for the function");

        assert_eq!(resource.name, "memory");
        assert_eq!(resource.value, "128MB");
        assert_eq!(
            resource.description,
            Some("Memory limit for the function".to_string())
        );
    }

    #[test]
    fn test_requirements_builder() {
        let requirements = Requirements::new()
            .recommend(Resource::new("memory", "128MB"))
            .recommend(Resource::new("timeout", "30s"))
            .require(Resource::new("cpu", "1x"))
            .platform("aws")
            .platform("cloudflare")
            .env_var("DATABASE_URL")
            .env_var("API_KEY");

        assert_eq!(requirements.recommended.len(), 2);
        assert_eq!(requirements.required.len(), 1);
        assert_eq!(requirements.platforms.len(), 2);
        assert_eq!(requirements.environment.len(), 2);

        assert!(requirements.get_recommended("memory").is_some());
        assert!(requirements.get_recommended("unknown").is_none());
        assert!(requirements.get_required("cpu").is_some());
        assert!(requirements.supports_platform("aws"));
        assert!(!requirements.supports_platform("azure"));
    }

    #[test]
    fn test_serialization() {
        let requirements = Requirements::new()
            .recommend(Resource::new("memory", "128MB"))
            .require(Resource::new("cpu", "1x"))
            .platform("aws")
            .env_var("API_KEY");

        let json = serde_json::to_string_pretty(&requirements).unwrap();
        let deserialized: Requirements = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.recommended.len(), 1);
        assert_eq!(deserialized.required.len(), 1);
        assert_eq!(deserialized.platforms.len(), 1);
        assert_eq!(deserialized.environment.len(), 1);
    }
}
