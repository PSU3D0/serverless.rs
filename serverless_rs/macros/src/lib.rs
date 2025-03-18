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
/// ```
/// use serverless_rs::{Request, Response, Context};
///
/// #[serverless]
/// async fn handler(req: Request, ctx: &Context) -> serverless_rs::Result<Response> {
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
/// ```
/// #[serverless(name = "api_handler", description = "API endpoint for user data")]
/// async fn handler(req: Request, ctx: &Context) -> serverless_rs::Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn serverless(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;

    // Parse attribute arguments
    let mut name = None;
    let mut description = None;
    let mut platforms = Vec::new();

    // A simpler implementation of attribute parsing since we're having issues with AttributeArgs
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
            // Simple stub implementation, will be expanded in a full version
            platforms.push("aws".to_string());
            platforms.push("cloudflare".to_string());
            return Ok(());
        }

        Ok(())
    };

    // Try to parse the arguments, but don't fail if there's an error
    let _ = syn::meta::parser(parser).parse(args.into());

    // Set default values
    let fn_name_str = name.unwrap_or_else(|| fn_name.to_string());
    let description_str =
        description.unwrap_or_else(|| format!("Serverless function {}", fn_name_str));

    // Generate the function information structure
    let info_struct = generate_info_struct(&fn_name_str, &description_str, &platforms);

    // Generate the platform-specific adapters
    let aws_adapter = generate_aws_adapter(&input_fn, &fn_name_str);
    let cloudflare_adapter = generate_cloudflare_adapter(&input_fn, &fn_name_str);
    let azure_adapter = generate_azure_adapter(&input_fn, &fn_name_str);
    let gcp_adapter = generate_gcp_adapter(&input_fn, &fn_name_str);
    let vercel_adapter = generate_vercel_adapter(&input_fn, &fn_name_str);
    let local_adapter = generate_local_adapter(&input_fn, &fn_name_str);

    // Generate the main handler implementation
    let expanded = quote! {
        // Preserve the original function
        #(#fn_attrs)*
        #fn_vis #input_fn

        // Define function information for self-documentation
        impl #fn_name {
            pub fn function_info() -> serverless_rs::FunctionInfo {
                #info_struct
            }

            // Check if this function was called with --info flag
            pub fn check_info() -> bool {
                serverless_rs::check_info_flag()
            }

            // Display function information
            pub fn display_info() {
                serverless_rs::display_info(&Self::function_info());
            }
        }

        // Platform-specific adapters
        #aws_adapter
        #cloudflare_adapter
        #azure_adapter
        #gcp_adapter
        #vercel_adapter
        #local_adapter
    };

    TokenStream::from(expanded)
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
            requirements = requirements.platform("aws");
            #[cfg(feature = "cloudflare")]
            requirements = requirements.platform("cloudflare");
            #[cfg(feature = "azure")]
            requirements = requirements.platform("azure");
            #[cfg(feature = "gcp")]
            requirements = requirements.platform("gcp");
            #[cfg(feature = "vercel")]
            requirements = requirements.platform("vercel");
            #[cfg(feature = "local")]
            requirements = requirements.platform("local");
        }
    } else {
        let platform_tokens = platforms.iter().map(|p| {
            let platform = p.as_str();
            quote! {
                requirements = requirements.platform(#platform);
            }
        });
        quote! {
            #(#platform_tokens)*
        }
    };

    quote! {
        let mut requirements = serverless_rs::Requirements::new();
        #platforms_tokens

        serverless_rs::FunctionInfo::new(#fn_name)
            .with_description(#description)
            .with_resources(requirements)
    }
}

/// Generate the AWS Lambda adapter
fn generate_aws_adapter(input_fn: &ItemFn, fn_name_str: &str) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[cfg(feature = "aws")]
        pub mod aws_lambda {
            use super::*;

            // This is a placeholder for the AWS Lambda adapter
            // It will be expanded in Step 4 per the execution plan
            pub extern "C" fn handler(event: serverless_rs::Value, context: serverless_rs::Value) -> serverless_rs::Value {
                // Check if the function was called with --info flag
                if #fn_name::check_info() {
                    #fn_name::display_info();
                    return serverless_rs::json!({
                        "statusCode": 200,
                        "body": "Function information displayed"
                    });
                }

                // Get the Lambda runtime and execute the handler
                // This is a simplified version for now
                let runtime = std::thread::spawn(move || {
                    let req = serverless_rs::Request::new()
                        .with_raw_event(event.clone());

                    let ctx = serverless_rs::Context::new()
                        .with_request_id("aws-request-id")
                        .with_function_name(#fn_name_str)
                        .with_platform_data(context);

                    match #fn_name(req, &ctx) {
                        Ok(resp) => {
                            serverless_rs::json!({
                                "statusCode": resp.status(),
                                "headers": resp.headers(),
                                "body": String::from_utf8_lossy(resp.body()).to_string(),
                                "isBase64Encoded": resp.is_base64()
                            })
                        },
                        Err(err) => {
                            serverless_rs::json!({
                                "statusCode": 500,
                                "body": format!("Error: {}", err)
                            })
                        }
                    }
                }).join().unwrap_or_else(|_| {
                    serverless_rs::json!({
                        "statusCode": 500,
                        "body": "Internal Error: Handler panicked"
                    })
                });

                runtime
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

                    match #fn_name(req, &ctx) {
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
                #fn_name(request, context)
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
/// ```
/// use serverless_rs::{Request, Response, Context, Result};
///
/// #[route(GET, "/hello")]
/// async fn hello_handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    // Parse attribute arguments manually - expecting HTTP method and path
    let args_span = proc_macro2::TokenStream::from(args);
    let args_str = args_span.to_string();

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

    // Generate route information
    let expanded = quote! {
        #input_fn

        impl #fn_name {
            pub fn route_info() -> serverless_rs::RouteInfo {
                serverless_rs::RouteInfo::new(#method, #path)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Requirements attribute macro for defining resource requirements
///
/// This macro will be fully implemented in Step 3 per the execution plan.
/// Currently it just parses basic requirements.
///
/// # Example
///
/// ```
/// use serverless_rs::{Request, Response, Context, Result};
///
/// #[requirements(
///     recommend(memory = "128MB", timeout = "30s"),
///     require(cpu = "1x")
/// )]
/// async fn handler(req: Request, ctx: &Context) -> Result<Response> {
///     Ok(Response::text("Hello, world!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn requirements(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    // Simple parsing of requirements - this is a stub for now
    // Will be fully implemented in Step 3
    let expanded = quote! {
        #input_fn

        // This is a stub implementation that will be expanded in Step 3
        impl #fn_name {
            pub fn has_requirements() -> bool {
                true
            }
        }
    };

    TokenStream::from(expanded)
}
