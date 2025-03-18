# serverless_rs Implementation Guide

This document outlines the implementation approach for `serverless_rs`, a universal serverless framework for Rust that enables deploying the same code to multiple serverless platforms with minimal platform-specific code.

## Core Principles

1. **Platform Agnostic Binaries**: The library focuses solely on creating platform-agnostic binaries via feature flags, without dictating specific storage implementations or other details.

2. **Multiple Execution Patterns**: Support both direct function patterns (implementing a `Handler` trait) and server patterns (supporting full HTTP verb routing like GET, POST, etc.).

3. **Resource Recommendation vs. Declaration**: The framework doesn't set resource requirements but allows annotating functions with "recommended" or "hard requirement" specifications for IaC tools to use.

4. **Self-documenting Functions**: Each compiled function can expose its recommended execution environment via a `--info` flag for use by IaC tools.

5. **IaC Integration**: Design with infrastructure-as-code integration in mind, allowing tools like Terraform to override recommendations while respecting hard requirements.

## Implementation Roadmap

### Phase 1: Core Framework Design

1. **Define Abstractions and Traits**
   - Create universal `Handler` trait for function handlers
   - Create `Router` trait for server patterns supporting HTTP methods
   - Define platform-agnostic request/response models
   - Create `Context` abstraction for accessing platform capabilities

2. **Implement Feature Flag Architecture**
   - Define clear feature flag structure for each platform
   - Ensure feature flags only control platform-specific code generation
   - Implement conditional compilation for each adapter

3. **Create Self-Documentation Mechanism**
   - Implement `--info` command-line flag
   - Define JSON output schema for resource recommendations
   - Create annotation system for declaring requirements

### Phase 2: Platform Adapters

1. **AWS Lambda Adapter**
   - Support direct function invocation pattern
   - Support API Gateway integration for server pattern
   - Implement context conversion
   - Support common trigger types (HTTP, EventBridge, etc.)

2. **Cloudflare Workers Adapter**
   - Support Workers runtime
   - Implement router for server pattern
   - Provide adapter for KV, D1, and other Cloudflare services

3. **Additional Platform Adapters**
   - Vercel Functions
   - Azure Functions
   - Google Cloud Functions

### Phase 3: IaC Integration

1. **Build Terraform Provider**
   - Create custom `serverless_rs_function` resource
   - Implement auto-discovery of function requirements
   - Support overriding recommendations while respecting hard requirements
   - Implement multi-platform deployment from single source

2. **CI/CD Pipeline Integration**
   - Create GitHub Actions for multi-platform builds
   - Implement testing framework for serverless functions
   - Create deployment workflows for major platforms

## Component Recommendations

### Core Library

```
serverless_rs/
├── src/
│   ├── lib.rs           # Public API and core abstractions
│   ├── handler.rs       # Handler trait definition
│   ├── router.rs        # Router for server pattern
│   ├── context.rs       # Universal context
│   ├── request.rs       # Request abstraction
│   ├── response.rs      # Response abstraction
│   ├── requirements.rs  # Resource requirement annotations
│   ├── info.rs          # Self-documentation mechanism
│   ├── platforms/       # Platform-specific adapters
│   │   ├── aws.rs       # AWS Lambda adapter
│   │   ├── cloudflare.rs # Cloudflare Workers adapter
│   │   └── ...
├── macros/             # Procedural macros
│   ├── src/
│   │   ├── lib.rs       # Export macros
│   │   ├── serverless.rs # Main #[serverless] macro
│   │   └── router.rs    # Router macro for server pattern
```

### Attribute Macro Design

The `#[serverless]` macro should:
- Accept function and server patterns
- Allow resource requirement annotations
- Generate platform-specific entry points only when feature flags are enabled
- Create info command handler

Example usage pattern (not actual code):

```rust
#[serverless]
#[requirements(
    recommend(memory = "128MB", timeout = "30s"),
    require(cpu = "1x")
)]
async fn handler(req: Request, ctx: Context) -> Response {
    // Function implementation
}
```

### Info Command Output

The `--info` flag should output JSON describing:
- Function name and description
- Recommended resources (memory, CPU, timeout)
- Hard requirements
- Supported platforms
- HTTP routes (for server pattern)
- Environment variables used

This output will be consumed by IaC tools.

## Terraform Integration

The Terraform provider should:
1. Locate serverless functions in a directory or repository
2. Compile them with appropriate feature flags
3. Run with `--info` to discover requirements
4. Generate appropriate resources for the target platform
5. Allow overriding recommendations in Terraform configuration

Example conceptual Terraform usage (not actual HCL):

```hcl
resource "serverless_rs_function" "api" {
  source_dir = "./src/functions/api"
  platforms = ["aws", "cloudflare"]
  
  override {
    memory = "256MB"
    timeout = "60s"
  }
}
```

## Testing Approach

1. **Unit Testing**: Test business logic separately from platform adapters
2. **Integration Testing**: Test platform adapters with mocked platform services
3. **Local Development**: Provide local server mode for testing
4. **Platform Testing**: Test deployed functions against actual platforms

## Documentation Requirements

1. **Core Documentation**: Document traits, macros, and abstractions
2. **Platform Guides**: Create guides for each supported platform
3. **IaC Integration**: Document Terraform provider and other IaC integrations
4. **Examples**: Provide examples for common use cases:
   - API Function
   - Background Processing
   - Schedule Tasks
   - Event Processing

## Design Considerations

1. **Performance**: Minimize runtime overhead
2. **Flexibility**: Don't force specific patterns or storage implementations
3. **Compatibility**: Work with existing platform tools and SDKs
4. **Extensibility**: Make it easy to add new platforms
5. **Development Experience**: Focus on reducing boilerplate and friction

## Next Steps

To implement this vision, follow these key steps:

1. **Define Core Abstractions**: Create the traits, context, and request/response models
2. **Implement Platform Adapters**: Build adapters for major platforms
3. **Create Terraform Provider**: Build custom provider for IaC integration
4. **Develop Documentation & Examples**: Create comprehensive guides and examples
5. **Set Up CI/CD & Testing**: Establish testing and deployment infrastructure

This implementation approach focuses on creating a flexible, platform-agnostic framework that integrates well with existing infrastructure tools while providing a unified development experience across serverless platforms.
