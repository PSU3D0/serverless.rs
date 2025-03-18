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

// #[serverless] - This will be available in the future
async fn handler(req: Request, ctx: &Context) -> Result<Response> {
    Ok(Response::new()
        .with_status(200)
        .with_body("Hello from serverless.rs!"))
}
```
*/

mod context;
mod error;
mod handler;
mod info;
mod platforms;
mod request;
mod requirements;
mod response;
mod router;

// Re-export main types
pub use context::Context;
pub use error::{Error, Result};
pub use handler::Handler;
pub use request::Request;
pub use requirements::{Requirements, Resource};
pub use response::Response;
pub use router::Router;

// Re-export macros (will be available after implementing the macros crate)
// #[cfg(feature = "macros")]
// pub use serverless_rs_macros::serverless;

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
