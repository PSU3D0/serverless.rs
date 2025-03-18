/*!
# serverless.rs

A universal serverless framework for Rust that enables writing platform-agnostic
serverless functions deployable to multiple cloud providers with minimal platform-specific code.

## Features

- Write once, deploy anywhere
- Type safety and performance optimization
- Infrastructure recommendations
- Seamless integration with CI/CD and IaC tools

## Example

```rust,no_run
use serverless_rs::{Context, Request, Response, Result};

// Use the #[serverless] attribute to transform your function
// (the macro is included but doctest will fail until fully implemented)
// #[serverless]
async fn handler(req: Request, ctx: &Context) -> Result<Response> {
    Ok(Response::new()
        .with_status(200)
        .with_body("Hello from serverless.rs!"))
}
```

## Platform Support

By default, the local development server is enabled. To deploy to a specific platform,
enable the corresponding feature flag:

- `aws` - AWS Lambda
- `cloudflare` - Cloudflare Workers
- `azure` - Azure Functions
- `gcp` - Google Cloud Functions
- `vercel` - Vercel Functions
- `local` - Local development server

## Attribute Macros

- `#[serverless]` - Mark a function as a serverless handler
- `#[route]` - Define an HTTP route
- `#[requirements]` - Specify resource requirements

## Resource Requirements

Use the `#[requirements]` attribute to specify recommended and required resources:

```rust,no_run
use serverless_rs::{Context, Request, Response, Result};

// Use the #[requirements] attribute to specify resources
// (the macro is included but doctest will fail until fully implemented)
// #[serverless]
// #[requirements(
//     recommend(memory = "128MB", timeout = "30s"),
//     require(cpu = "1x")
// )]
async fn handler(req: Request, ctx: &Context) -> Result<Response> {
    Ok(Response::text("Hello, world!"))
}
```
*/

mod context;
mod error;
mod handler;
mod info;
pub mod platforms;
mod request;
mod requirements;
mod response;
mod router;

// Re-export main types
pub use context::Context;
pub use error::{Error, Result};
pub use handler::Handler;
pub use info::{
    check_info_flag, display_info, handle_info_request, parse_info_args, FunctionInfo,
    OutputFormat, RouteInfo,
};
pub use request::Request;
pub use requirements::{Requirements, Resource};
pub use response::Response;
pub use router::Router;

// Re-export macros
pub use serverless_rs_macros::{requirements, route, serverless};

// Re-export serde_json for use in macros
pub use serde_json::{json, Value};

/// Version of the serverless.rs framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
