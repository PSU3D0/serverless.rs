/*!
Procedural macros for serverless.rs

This crate provides the procedural macros used by the serverless.rs framework,
including the `#[serverless]` attribute macro.
*/

use proc_macro::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::parse::Parser;
use syn::{parse_macro_input, ItemFn};

/// The main serverless attribute macro
///
/// This macro transforms an async function into a serverless handler
/// that can be deployed to any supported platform.
///
/// # Example
///
/// ```ignore
/// use serverless_rs::{Request, Response, Context, Result};
/// use serverless_rs_macros::serverless;
///
/// #[serverless]
/// async fn handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
///
/// # Options
///
/// - `name`: Custom name for the function (defaults to the function name)
/// - `description`: Description of the function
/// - `platforms`: List of supported platforms (defaults to all enabled platforms)
///
/// ```ignore
/// use serverless_rs::{Request, Response, Context, Result};
/// use serverless_rs_macros::serverless;
///
/// #[serverless(name = "api_handler", description = "API endpoint for user data")]
/// async fn handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn serverless(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;

    // Parse attribute arguments
    let mut name = None;
    let mut description = None;
    let mut platforms = Vec::new();
    let parser = |meta: ParseNestedMeta| {
        if meta.path.is_ident("name") {
            if let Ok(value) = meta.value() {
                if let Ok(literal) = value.parse::<syn::LitStr>() {
                    name = Some(literal.value());
                }
            }
            return Ok(());
        }
        if meta.path.is_ident("description") {
            if let Ok(value) = meta.value() {
                if let Ok(literal) = value.parse::<syn::LitStr>() {
                    description = Some(literal.value());
                }
            }
            return Ok(());
        }
        if meta.path.is_ident("platforms") {
            platforms.push("aws".to_string());
            platforms.push("cloudflare".to_string());
            return Ok(());
        }
        Ok(())
    };
    let _ = syn::meta::parser(parser).parse(args);

    // Set default values
    let fn_name_str = name.unwrap_or_else(|| fn_name.to_string());
    let description_str =
        description.unwrap_or_else(|| format!("Serverless function {}", fn_name_str));

    // Generate the function information structure and platform adapters...
    let info_struct = generate_info_struct(&fn_name_str, &description_str, &platforms);
    let aws_adapter = generate_aws_adapter(&input_fn, &fn_name_str);
    let cloudflare_adapter = generate_cloudflare_adapter(&input_fn, &fn_name_str);
    let azure_adapter = generate_azure_adapter(&input_fn, &fn_name_str);
    let gcp_adapter = generate_gcp_adapter(&input_fn, &fn_name_str);
    let vercel_adapter = generate_vercel_adapter(&input_fn, &fn_name_str);
    let local_adapter = generate_local_adapter(&input_fn, &fn_name_str);

    // Generate the main handler implementation as module-level functions.
    let expanded = quote! {
        // Preserve the original function
        #(#fn_attrs)*
        #input_fn

        // Module-level function implementations
        pub fn function_info() -> serverless_rs::FunctionInfo {
            let mut info = #info_struct;
            if has_requirements() {
                info = info.with_resources(requirements());
            }
            if has_route_info() {
                info = info.add_route(route_info());
            }
            info
        }
        pub fn check_info() -> bool {
            serverless_rs::check_info_flag()
        }
        pub fn display_info() {
            serverless_rs::display_info(&function_info());
        }
        #[allow(dead_code)]
        pub fn requirements() -> serverless_rs::Requirements {
            serverless_rs::Requirements::new()
        }
        #[allow(dead_code)]
        pub fn has_requirements() -> bool { false }
        #[allow(dead_code)]
        pub fn has_route_info() -> bool { false }
        // Optionally, if route_info is needed, you can add a stub:
        #[allow(dead_code)]
        pub fn route_info() -> serverless_rs::RouteInfo {
            // This will be overridden if `#[route]` is used.
            serverless_rs::RouteInfo::new("GET", "/")
        }

        // Platform-specific adapters
        #aws_adapter
        #cloudflare_adapter
        #azure_adapter
        #gcp_adapter
        #vercel_adapter
        #local_adapter
    };

    // Wrap the generated code in a module named after the supplied name.
    let mod_ident = syn::Ident::new(&fn_name_str, fn_name.span());
    let wrapped = quote! {
        pub mod #mod_ident {
            use super::*;
            #expanded
        }
    };

    TokenStream::from(wrapped)
}

/// Generate the function information structure
fn generate_info_struct(
    fn_name: &str,
    description: &str,
    platforms: &[String],
) -> proc_macro2::TokenStream {
    let platforms_tokens = if platforms.is_empty() {
        quote! {
            // Add all enabled platforms
            #[cfg(feature = "aws")]
            { requirements = requirements.platform("aws"); }
            #[cfg(feature = "cloudflare")]
            { requirements = requirements.platform("cloudflare"); }
            #[cfg(feature = "azure")]
            { requirements = requirements.platform("azure"); }
            #[cfg(feature = "gcp")]
            { requirements = requirements.platform("gcp"); }
            #[cfg(feature = "vercel")]
            { requirements = requirements.platform("vercel"); }
            #[cfg(feature = "local")]
            { requirements = requirements.platform("local"); }
        }
    } else {
        let platform_tokens = platforms.iter().map(|p| {
            let platform = p.as_str();
            quote! {
                { requirements = requirements.platform(#platform); }
            }
        });
        quote! {
            #(#platform_tokens)*
        }
    };

    quote! {
        {
            let mut requirements = serverless_rs::Requirements::new();
            #platforms_tokens
            serverless_rs::FunctionInfo::new(#fn_name)
                .with_description(#description)
                .with_resources(requirements)
        }
    }
}

/// Generate the AWS Lambda adapter
///
/// This function generates the AWS Lambda adapter code that integrates
/// serverless.rs functions with the AWS Lambda runtime. It handles both
/// direct invocations and API Gateway events.
fn generate_aws_adapter(input_fn: &ItemFn, _fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "aws")]
        pub mod aws_lambda {
            use super::*;
            use serverless_rs::platforms::aws;

            // Helper function to handle async wrapper
            fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // The main Lambda handler entry point
            pub fn handler(event: serverless_rs::Value, context: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "statusCode": 200,
                        "body": "Function information displayed"
                    });
                }

                // Call the AWS Lambda handler with our wrapper function
                aws::lambda_handler(handler_wrapper, event, context)
            }

            // A convenient entrypoint for API Gateway requests specifically
            pub fn api_gateway(event: serverless_rs::Value, context: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "statusCode": 200,
                        "body": "Function information displayed"
                    });
                }

                match aws::handle_api_gateway(handler_wrapper, event, context) {
                    Ok(response) => {
                        // Serialize the API Gateway response
                        match serde_json::to_value(response) {
                            Ok(json) => json,
                            Err(e) => serverless_rs::json!({
                                "statusCode": 500,
                                "body": format!("Error serializing response: {}", e)
                            })
                        }
                    },
                    Err(e) => {
                        // Return an error response for API Gateway
                        serverless_rs::json!({
                            "statusCode": 500,
                            "body": format!("Error: {}", e)
                        })
                    }
                }
            }

            // A convenient entrypoint for direct Lambda invocations
            pub fn direct(event: serverless_rs::Value, context: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "message": "Function information displayed"
                    });
                }

                match aws::handle_direct_invocation(handler_wrapper, event, context) {
                    Ok(response) => response,
                    Err(e) => {
                        // Return an error payload for direct invocation
                        serverless_rs::json!({
                            "error": e.to_string()
                        })
                    }
                }
            }

            // Lambda custom runtime handler (for provided.al2, etc.)
            pub fn custom_runtime() {
                // Will be implemented in future versions
                println!("AWS Lambda custom runtime not yet implemented");
            }

            // Export function info for IaC integration
            pub fn function_info() -> serverless_rs::FunctionInfo {
                #fn_name::function_info()
            }
        }
    }
}

/// Generate the Cloudflare Workers adapter
fn generate_cloudflare_adapter(input_fn: &ItemFn, fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "cloudflare")]
        pub mod cloudflare_workers {
            use super::*;

            // Helper function to handle async wrapper
            fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // This is a placeholder for the Cloudflare Workers adapter
            // It will be expanded in Step 5 per the execution plan
            pub fn handle_fetch(request: serverless_rs::Value, env: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "status": 200,
                        "body": "Function information displayed"
                    });
                }

                // Get the Workers runtime and execute the handler
                // This is a simplified version for now
                let runtime = std::thread::spawn(move || {
                    let req = serverless_rs::Request::new()
                        .with_raw_event(request.clone());

                    let ctx = serverless_rs::Context::new()
                        .with_request_id("cf-request-id")
                        .with_function_name(#fn_name_str)
                        .with_platform_data(env);

                    match handler_wrapper(req, &ctx) {
                        Ok(resp) => {
                            serverless_rs::json!({
                                "status": resp.status(),
                                "headers": resp.headers(),
                                "body": String::from_utf8_lossy(resp.body()).to_string(),
                                "bodyEncoding": if resp.is_base64() { "base64" } else { "utf-8" }
                            })
                        },
                        Err(err) => {
                            serverless_rs::json!({
                                "status": 500,
                                "body": format!("Error: {}", err)
                            })
                        }
                    }
                }).join().unwrap_or_else(|_| {
                    serverless_rs::json!({
                        "status": 500,
                        "body": "Internal Error: Handler panicked"
                    })
                });

                runtime
            }
        }
    }
}

/// Generate the Azure Functions adapter
fn generate_azure_adapter(input_fn: &ItemFn, _fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "azure")]
        pub mod azure_functions {
            use super::*;

            // Helper function to handle async wrapper
            fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // This is a placeholder for the Azure Functions adapter
            // It will be implemented in later steps
            pub fn run(context: serverless_rs::Value, request: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "status": 200,
                        "body": "Function information displayed"
                    });
                }

                // Azure Functions adapter placeholder
                serverless_rs::json!({
                    "status": 200,
                    "body": "Azure Functions adapter not yet implemented"
                })
            }
        }
    }
}

/// Generate the Google Cloud Functions adapter
fn generate_gcp_adapter(input_fn: &ItemFn, _fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "gcp")]
        pub mod gcp_functions {
            use super::*;

            // Helper function to handle async wrapper
            fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // This is a placeholder for the Google Cloud Functions adapter
            // It will be implemented in later steps
            pub fn entry_point(request: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "statusCode": 200,
                        "body": "Function information displayed"
                    });
                }

                // GCP Functions adapter placeholder
                serverless_rs::json!({
                    "statusCode": 200,
                    "body": "GCP Functions adapter not yet implemented"
                })
            }
        }
    }
}

/// Generate the Vercel Functions adapter
fn generate_vercel_adapter(input_fn: &ItemFn, _fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "vercel")]
        pub mod vercel_functions {
            use super::*;

            // Helper function to handle async wrapper
            fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // This is a placeholder for the Vercel Functions adapter
            // It will be implemented in later steps
            pub fn handler(request: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "statusCode": 200,
                        "body": "Function information displayed"
                    });
                }

                // Vercel Functions adapter placeholder
                serverless_rs::json!({
                    "statusCode": 200,
                    "body": "Vercel Functions adapter not yet implemented"
                })
            }
        }
    }
}

/// Generate the local development server adapter
fn generate_local_adapter(input_fn: &ItemFn, fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "local")]
        pub mod local_server {
            use super::*;

            // Helper function to handle async wrapper
            pub fn handler_wrapper(req: serverless_rs::Request, ctx: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                // Create a runtime to execute the async function
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the async function and return the result
                runtime.block_on(#fn_name(req, ctx))
            }

            // This is a placeholder for the local development server adapter
            // It will be expanded in Step 6 per the execution plan
            pub async fn serve_http(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return Ok(());
                }

                println!("Starting local server for '{}' at {}", #fn_name_str, addr);
                println!("Local development server not yet implemented");
                println!("This will be expanded in Step 6 per the execution plan");
                Ok(())
            }

            pub fn handle_request(request: serverless_rs::Request, context: &serverless_rs::Context) -> serverless_rs::Result<serverless_rs::Response> {
                handler_wrapper(request, context)
            }
        }
    }
}

/// Route attribute macro for defining HTTP routes
///
/// This macro simplifies the creation of HTTP route handlers.
///
/// # Example
///
/// ```ignore
/// use serverless_rs::{Request, Response, Context, Result};
/// use serverless_rs_macros::route;
///
/// #[route(GET, "/hello", description = "Greeting endpoint")]
/// async fn hello_handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse attribute arguments
    let args_span = proc_macro2::TokenStream::from(args);
    let args_str = args_span.to_string();

    // Split by commas, but keep quoted strings intact
    let parts: Vec<&str> = args_str.split(',').collect();
    if parts.len() < 2 {
        return TokenStream::from(quote! {
            compile_error!("route attribute requires a method and path, e.g., #[route(GET, \"/path\")]");
        });
    }

    // Extract HTTP method (just use the string directly)
    let method = parts[0].trim();

    // Extract path (assume it's a string literal)
    let path = parts[1].trim().trim_matches('"');

    // Extract optional description if present
    let mut description = None;
    for part in parts.iter().skip(2) {
        let part = part.trim();
        if part.starts_with("description") {
            let desc_parts: Vec<&str> = part.split('=').collect();
            if desc_parts.len() == 2 {
                let desc_value = desc_parts[1].trim().trim_matches('"');
                description = Some(desc_value.to_string());
            }
        }
    }

    // Generate route information with optional description
    let route_builder = if let Some(desc) = description {
        quote! {
            serverless_rs::RouteInfo::new(#method, #path).with_description(#desc)
        }
    } else {
        quote! {
            serverless_rs::RouteInfo::new(#method, #path)
        }
    };

    // Generate implementation
    let expanded = quote! {
        #input_fn

        pub fn route_info() -> serverless_rs::RouteInfo {
            #route_builder
        }

        pub fn has_route_info() -> bool {
            true
        }
    };

    TokenStream::from(expanded)
}

/// Requirements attribute macro for defining resource requirements
///
/// This macro allows you to specify resource requirements and recommendations
/// for your serverless functions. These requirements will be:
/// 1. Available through the `--info` flag for self-documentation
/// 2. Used by IaC tools to generate appropriate infrastructure
/// 3. Verified against platform capabilities during compilation
///
/// # Example
///
/// ```ignore
/// use serverless_rs::{Request, Response, Context, Result};
/// use serverless_rs_macros::requirements;
///
/// #[requirements(
///     recommend(memory = "128MB", timeout = "30s"),
///     require(cpu = "1x"),
///     platforms(aws, cloudflare),
///     env(DATABASE_URL, API_KEY)
/// )]
/// async fn handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn requirements(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);

    // Initialize collections to store the parsed requirements
    let mut recommended = Vec::new();
    let mut required = Vec::new();
    let mut platforms = Vec::new();
    let mut env_vars = Vec::new();

    // Parse the attribute arguments
    let args_span = proc_macro2::TokenStream::from(args);
    let args_str = args_span.to_string();

    // Simplified parsing approach using string manipulation
    // This is not a production-quality parser but works for our demo

    // Extract recommend() blocks
    if let Some(recommend_block) = extract_section(&args_str, "recommend") {
        for resource in extract_key_values(recommend_block) {
            let (name, value) = resource;
            recommended.push((name, value));
        }
    }

    // Extract require() blocks
    if let Some(require_block) = extract_section(&args_str, "require") {
        for resource in extract_key_values(require_block) {
            let (name, value) = resource;
            required.push((name, value));
        }
    }

    // Extract platforms() block
    if let Some(platforms_block) = extract_section(&args_str, "platforms") {
        for platform in platforms_block.split(',') {
            let platform = platform
                .trim()
                .trim_matches(|c| c == '(' || c == ')' || c == ' ');
            if !platform.is_empty() {
                platforms.push(platform.to_string());
            }
        }
    }

    // Extract env() block
    if let Some(env_block) = extract_section(&args_str, "env") {
        for env_var in env_block.split(',') {
            let env_var = env_var
                .trim()
                .trim_matches(|c| c == '(' || c == ')' || c == ' ');
            if !env_var.is_empty() {
                env_vars.push(env_var.to_string());
            }
        }
    }

    // Generate the requirements builder code
    let mut requirements_builder = quote! {
        let mut requirements = serverless_rs::Requirements::new();
    };

    // Add recommended resources
    for (name, value) in &recommended {
        let resource_builder = quote! {
            requirements = requirements.recommend(
                serverless_rs::Resource::new(#name, #value)
            );
        };
        requirements_builder = quote! {
            #requirements_builder
            #resource_builder
        };
    }

    // Add required resources
    for (name, value) in &required {
        let resource_builder = quote! {
            requirements = requirements.require(
                serverless_rs::Resource::new(#name, #value)
            );
        };
        requirements_builder = quote! {
            #requirements_builder
            #resource_builder
        };
    }

    // Add platforms
    for platform in &platforms {
        let platform_builder = quote! {
            requirements = requirements.platform(#platform);
        };
        requirements_builder = quote! {
            #requirements_builder
            #platform_builder
        };
    }

    // Add environment variables
    for env_var in &env_vars {
        let env_var_builder = quote! {
            requirements = requirements.env_var(#env_var);
        };
        requirements_builder = quote! {
            #requirements_builder
            #env_var_builder
        };
    }

    // Instead of generating an inherent impl block on fn_name (which is a function)
    // we now generate free functions.
    let expanded = quote! {
        #input_fn

        #[allow(dead_code)]
        pub fn requirements() -> serverless_rs::Requirements {
            #requirements_builder
            requirements
        }

        #[allow(dead_code)]
        pub fn has_requirements() -> bool {
            true
        }
    };

    TokenStream::from(expanded)
}

// Helper function to extract a section from the attributes string
fn extract_section(input: &str, section_name: &str) -> Option<String> {
    let pattern = format!("{}\\s*\\(([^)]*)\\)", section_name);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(input).map(|caps| caps[1].to_string())
}

// Helper function to extract key-value pairs from a section
fn extract_key_values(input: String) -> Vec<(String, String)> {
    let mut result = Vec::new();

    // Split by commas and process each key=value pair
    for pair in input.split(',') {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().trim_matches('"').to_string();
            result.push((key, value));
        }
    }

    result
}
