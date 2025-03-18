# serverless.rs

A universal serverless framework for Rust that enables developers to write platform-agnostic serverless functions and deploy them to multiple cloud providers with minimal platform-specific code.

## Project Status

🚧 **Early Development** 🚧

This project is in the early stages of development. The core abstractions are being defined, but the framework is not yet ready for production use.

## Features

- **Write once, deploy anywhere**: Build serverless functions that can run on multiple platforms without rewriting code.
- **Type safety and performance optimization**: Maintain full Rust type safety and performance characteristics.
- **Intelligent infrastructure recommendations**: Get smart recommendations for resource allocation.
- **Seamless integration with CI/CD and IaC tools**: Deploy easily with your existing tools.

## Design Goals

- Zero runtime overhead compared to native platform implementations
- Type-safe abstractions for serverless function development
- Clear separation between platform-agnostic and platform-specific code
- Comprehensive documentation and examples

## Supported Platforms (Planned)

- AWS Lambda
- Cloudflare Workers
- Vercel Functions
- Azure Functions
- Google Cloud Functions
- Local development server

## Example (Planned API)

```rust
use serverless_rs::{serverless, Request, Response, Context, Result};

#[serverless]
#[requirements(
    recommend(memory = "128MB", timeout = "30s"),
    require(cpu = "1x")
)]
async fn handler(req: Request, ctx: &Context) -> Result<Response> {
    // Function implementation
    Ok(Response::text("Hello from serverless.rs!"))
}
```

## Installation

_Coming soon_

## Documentation

_Coming soon_

## License

This project is licensed under the MIT License - see the LICENSE file for details.