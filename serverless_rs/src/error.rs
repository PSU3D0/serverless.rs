/*!
Error types for serverless.rs
*/

use std::fmt;
use thiserror::Error;

/// Error type for serverless.rs
#[derive(Error, Debug)]
pub enum Error {
    /// Error when serializing or deserializing data
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error when handling HTTP requests or responses
    #[error("HTTP error: {0}")]
    Http(String),

    /// Error from a specific platform implementation
    #[error("Platform error: {0}")]
    Platform(String),

    /// Error from user code
    #[error("Function error: {0}")]
    Function(String),

    /// Error when validating requirements
    #[error("Requirements error: {0}")]
    Requirements(String),

    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl Error {
    /// Creates a new serialization error
    pub fn serialization<T: fmt::Display>(err: T) -> Self {
        Self::Serialization(err.to_string())
    }

    /// Creates a new HTTP error
    pub fn http<T: fmt::Display>(err: T) -> Self {
        Self::Http(err.to_string())
    }

    /// Creates a new platform error
    pub fn platform<T: fmt::Display>(err: T) -> Self {
        Self::Platform(err.to_string())
    }

    /// Creates a new function error
    pub fn function<T: fmt::Display>(err: T) -> Self {
        Self::Function(err.to_string())
    }

    /// Creates a new requirements error
    pub fn requirements<T: fmt::Display>(err: T) -> Self {
        Self::Requirements(err.to_string())
    }

    /// Creates a new unexpected error
    pub fn unexpected<T: fmt::Display>(err: T) -> Self {
        Self::Unexpected(err.to_string())
    }
}

/// Result type for serverless.rs
pub type Result<T> = std::result::Result<T, Error>;
