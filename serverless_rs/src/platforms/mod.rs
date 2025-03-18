/*!
Platform-specific adapters for serverless.rs.

This module contains adapters for different serverless platforms.
*/

// AWS Lambda adapter
#[cfg(feature = "aws")]
pub mod aws;

// Cloudflare Workers adapter
#[cfg(feature = "cloudflare")]
pub mod cloudflare;

// Vercel Functions adapter
#[cfg(feature = "vercel")]
pub mod vercel;

// Azure Functions adapter
#[cfg(feature = "azure")]
pub mod azure;

// Google Cloud Functions adapter
#[cfg(feature = "gcp")]
pub mod gcp;

// Local development server
#[cfg(feature = "local")]
pub mod local;
