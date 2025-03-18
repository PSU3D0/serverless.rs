# CLAUDE.md - Development Guidelines

## Project Context

serverless.rs is a universal serverless framework for Rust that enables developers to write platform-agnostic serverless functions and deploy them to multiple cloud providers with minimal platform-specific code. This framework addresses the challenge of serverless vendor lock-in by providing a unified abstraction layer.

Follow `ai_specs/EXECUTION_PLAN.md` and `ai_specs/PROJECT_PRD.md`.

## Commands
- Build: `cargo build`
- Check: `cargo check --all-features`
- Test all: `cargo test --all-features`
- Test single: `cargo test --all-features [test_name]`
- Test specific module: `cargo test --all-features [module]::[test_name]`
- Lint: `cargo clippy --all-features -- -D warnings`
- Format: `cargo fmt --all`
- Run feature-specific tests: `cargo test --features=[feature_name]`
- Documentation: `cargo doc --no-deps --open`

## Style Guidelines
- Use 4-space indentation
- Follow Rust's naming conventions: snake_case for functions/variables, CamelCase for types
- Group imports: std first, then external crates, then local modules
- Prefer `?` operator for error handling and use `thiserror` for error types
- Ensure all public API has doc comments (including examples)
- For platform-specific code, always wrap in feature gates via `#[cfg(feature = "...")]`
- Use async/await for asynchronous code with proper error propagation
- Use type annotations for improved readability, especially in function signatures
- **IMPORTANT:** If following a PRD, include docstring-compliant comments for what parts of the PRD a struct/method/module fulfills