/*!
Platform-agnostic request model for serverless.rs.

This module provides a unified request abstraction that works across
different serverless platforms.
*/

use http::{Method, Uri};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A platform-agnostic request that can be handled by serverless functions
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method if this is an HTTP request
    method: Option<Method>,

    /// Request URI if this is an HTTP request
    uri: Option<Uri>,

    /// Request headers
    headers: HashMap<String, String>,

    /// Query parameters parsed from the URI
    query: HashMap<String, String>,

    /// Path parameters extracted from route patterns (e.g., /users/{id})
    path_params: HashMap<String, String>,

    /// Request body as raw bytes
    body: Vec<u8>,

    /// Original platform-specific event data
    raw_event: Value,
}

impl Request {
    /// Creates a new empty request
    pub fn new() -> Self {
        Self {
            method: None,
            uri: None,
            headers: HashMap::new(),
            query: HashMap::new(),
            path_params: HashMap::new(),
            body: Vec::new(),
            raw_event: Value::Null,
        }
    }

    /// Returns the HTTP method for this request, if available
    pub fn method(&self) -> Option<&Method> {
        self.method.as_ref()
    }

    /// Returns the HTTP method as a string, if available
    pub fn method_str(&self) -> Option<String> {
        self.method.as_ref().map(|m| m.to_string())
    }

    /// Sets the HTTP method for this request
    pub fn with_method(mut self, method: impl Into<Method>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Sets the HTTP method for this request using a string
    pub fn with_method_str(mut self, method: impl AsRef<str>) -> Self {
        if let Ok(m) = Method::from_str(method.as_ref()) {
            self.method = Some(m);
        }
        self
    }

    /// Returns the URI for this request, if available
    pub fn uri(&self) -> Option<&Uri> {
        self.uri.as_ref()
    }

    /// Returns the path portion of the URI, if available
    pub fn path(&self) -> Option<String> {
        self.uri.as_ref().map(|u| u.path().to_string())
    }

    /// Sets the URI for this request
    pub fn with_uri(mut self, uri: Uri) -> Self {
        self.uri = Some(uri);
        self
    }

    /// Sets the URI for this request using a string
    pub fn with_path(mut self, path: impl AsRef<str>) -> Self {
        if let Ok(uri) = Uri::from_str(path.as_ref()) {
            self.uri = Some(uri);
        }
        self
    }

    /// Returns the headers for this request
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Sets a header for this request
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Returns a header value by name
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Returns the query parameters for this request
    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }

    /// Sets a query parameter for this request
    pub fn with_query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(name.into(), value.into());
        self
    }

    /// Returns a query parameter by name
    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }

    /// Returns the path parameters for this request
    pub fn path_params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Sets a path parameter for this request
    pub fn with_path_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.path_params.insert(name.into(), value.into());
        self
    }

    /// Returns a path parameter by name
    pub fn path_param(&self, name: &str) -> Option<&String> {
        self.path_params.get(name)
    }

    /// Returns the raw body bytes for this request
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Sets the body for this request
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Returns the body as a string if it's valid UTF-8
    pub fn body_string(&self) -> Result<String> {
        String::from_utf8(self.body.clone()).map_err(Error::serialization)
    }

    /// Parse the body as JSON into the given type
    pub fn body_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(Error::serialization)
    }

    /// Returns the raw event data for this request
    pub fn raw_event(&self) -> &Value {
        &self.raw_event
    }

    /// Sets the raw event for this request
    pub fn with_raw_event(mut self, event: Value) -> Self {
        self.raw_event = event;
        self
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Method;

    #[test]
    fn test_request_builder() {
        let req = Request::new()
            .with_method(Method::GET)
            .with_uri("/api/users".parse().unwrap())
            .with_header("Content-Type", "application/json")
            .with_query("page", "1")
            .with_path_param("id", "123")
            .with_body(r#"{"name":"test"}"#);

        assert_eq!(req.method(), Some(&Method::GET));
        assert_eq!(req.uri().unwrap().to_string(), "/api/users");
        assert_eq!(
            req.header("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(req.query_param("page"), Some(&"1".to_string()));
        assert_eq!(req.path_param("id"), Some(&"123".to_string()));
        assert_eq!(req.body_string().unwrap(), r#"{"name":"test"}"#);
    }

    #[test]
    fn test_body_json() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestData {
            name: String,
        }

        let req = Request::new().with_body(r#"{"name":"test"}"#);

        let data: TestData = req.body_json().unwrap();
        assert_eq!(
            data,
            TestData {
                name: "test".to_string()
            }
        );
    }
}
