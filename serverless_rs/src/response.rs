/*!
Platform-agnostic response model for serverless.rs.

This module provides a unified response abstraction that works across
different serverless platforms.
*/

use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};

/// A platform-agnostic response from serverless functions
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code
    status: u16,

    /// Response headers
    headers: HashMap<String, String>,

    /// Response body as raw bytes
    body: Vec<u8>,

    /// Whether the response is Base64 encoded
    is_base64: bool,
}

impl Response {
    /// Creates a new response with default values (200 OK, empty body)
    pub fn new() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
            is_base64: false,
        }
    }

    /// Returns the status code for this response
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Sets the status code for this response
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Returns the headers for this response
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Sets a header for this response
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Returns a header value by name
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Returns the raw body bytes for this response
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Sets the body for this response
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Returns whether the body is Base64 encoded
    pub fn is_base64(&self) -> bool {
        self.is_base64
    }

    /// Sets whether the body is Base64 encoded
    pub fn with_base64(mut self, is_base64: bool) -> Self {
        self.is_base64 = is_base64;
        self
    }

    /// Creates a response with a JSON body
    pub fn json<T: Serialize>(value: &T) -> Result<Self> {
        let body = serde_json::to_vec(value).map_err(Error::serialization)?;

        Ok(Self::new()
            .with_header("Content-Type", "application/json")
            .with_body(body))
    }

    /// Creates a response with a text body
    pub fn text<T: AsRef<str>>(text: T) -> Self {
        Self::new()
            .with_header("Content-Type", "text/plain")
            .with_body(text.as_ref().as_bytes().to_vec())
    }

    /// Creates a response with an HTML body
    pub fn html<T: AsRef<str>>(html: T) -> Self {
        Self::new()
            .with_header("Content-Type", "text/html")
            .with_body(html.as_ref().as_bytes().to_vec())
    }

    /// Creates a redirect response
    pub fn redirect(location: impl Into<String>) -> Self {
        Self::new()
            .with_status(302)
            .with_header("Location", location.into())
    }

    /// Creates a "not found" response
    pub fn not_found() -> Self {
        Self::new().with_status(404).with_body("Not Found")
    }

    /// Creates a "bad request" response
    pub fn bad_request() -> Self {
        Self::new().with_status(400).with_body("Bad Request")
    }

    /// Creates an "internal server error" response
    pub fn internal_error() -> Self {
        Self::new()
            .with_status(500)
            .with_body("Internal Server Error")
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_response_builder() {
        let resp = Response::new()
            .with_status(201)
            .with_header("Content-Type", "application/json")
            .with_body(r#"{"id":123}"#);

        assert_eq!(resp.status(), 201);
        assert_eq!(
            resp.header("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(std::str::from_utf8(resp.body()).unwrap(), r#"{"id":123}"#);
    }

    #[test]
    fn test_json_response() {
        let data = json!({
            "id": 123,
            "name": "test"
        });

        let resp = Response::json(&data).unwrap();
        assert_eq!(
            resp.header("Content-Type"),
            Some(&"application/json".to_string())
        );

        let body_str = std::str::from_utf8(resp.body()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body_str).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    fn test_convenience_responses() {
        let text = Response::text("Hello, world!");
        assert_eq!(text.status(), 200);
        assert_eq!(text.header("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(std::str::from_utf8(text.body()).unwrap(), "Hello, world!");

        let html = Response::html("<h1>Hello</h1>");
        assert_eq!(html.status(), 200);
        assert_eq!(html.header("Content-Type"), Some(&"text/html".to_string()));
        assert_eq!(std::str::from_utf8(html.body()).unwrap(), "<h1>Hello</h1>");

        let redirect = Response::redirect("/dashboard");
        assert_eq!(redirect.status(), 302);
        assert_eq!(redirect.header("Location"), Some(&"/dashboard".to_string()));

        let not_found = Response::not_found();
        assert_eq!(not_found.status(), 404);

        let bad_request = Response::bad_request();
        assert_eq!(bad_request.status(), 400);

        let internal_error = Response::internal_error();
        assert_eq!(internal_error.status(), 500);
    }
}
