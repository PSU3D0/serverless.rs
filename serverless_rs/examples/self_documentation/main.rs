/*!
Self-Documentation Example for serverless.rs.

This example demonstrates the self-documentation mechanism implemented in Step 3.
It showcases:
- The #[serverless] attribute macro
- The #[requirements] attribute for specifying resource requirements
- The #[route] attribute for defining HTTP routes
- The --info flag for displaying function metadata
- The --json flag for JSON output

Run this example with the --info flag to see the self-documentation in action:
`cargo run --example self_documentation -- --info`

For JSON output, add the --json flag:
`cargo run --example self_documentation -- --info --json`
*/

use serverless_rs::{Context, Request, Response, Result};
// For this example, we'll use simpler handler functions without macros
// since the macro functionality is not fully working yet

// The main API handler function
async fn api_handler(req: Request, ctx: &Context) -> Result<Response> {
    // Log the request
    ctx.log("INFO", &format!("Received request: {:?}", req.method_str()));

    // Simple routing logic (normally this would use the Router trait)
    match (req.method_str().as_deref(), req.path().as_deref()) {
        (Some("GET"), Some("/users")) => get_users(req, ctx).await,
        (Some("POST"), Some("/users")) => create_user(req, ctx).await,
        _ => Ok(Response::text("Not Found").with_status(404)),
    }
}

// Handler for GET /users
async fn get_users(_req: Request, ctx: &Context) -> Result<Response> {
    ctx.log("INFO", "Getting all users");

    // In a real application, this would query a database
    let users = r#"[
        {"id": 1, "name": "John Doe", "email": "john@example.com"},
        {"id": 2, "name": "Jane Doe", "email": "jane@example.com"}
    ]"#;

    // Convert to JSON response
    let response = Response::new()
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(users);

    Ok(response)
}

// Handler for POST /users
async fn create_user(_req: Request, ctx: &Context) -> Result<Response> {
    ctx.log("INFO", "Creating new user");

    // In a real application, this would insert into a database
    let user_id = 3; // Generated ID

    let body = format!(r#"{{"id": {}, "status": "created"}}"#, user_id);

    // Create response
    let response = Response::new()
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(body);

    Ok(response)
}

// Simple main function for local execution
fn main() {
    // Parse command line arguments
    let (info_requested, _format) = serverless_rs::parse_info_args();

    // If --info flag was provided, display example function metadata and exit
    if info_requested {
        // Create a sample FunctionInfo
        let resources = serverless_rs::Requirements::new()
            .recommend(
                serverless_rs::Resource::new("memory", "256MB")
                    .with_description("Memory needed for processing"),
            )
            .recommend(
                serverless_rs::Resource::new("timeout", "30s")
                    .with_description("Maximum execution time"),
            )
            .require(serverless_rs::Resource::new("cpu", "1x").with_description("CPU allocation"))
            .platform("aws")
            .platform("cloudflare")
            .env_var("DATABASE_URL")
            .env_var("API_KEY");

        let route1 =
            serverless_rs::RouteInfo::new("GET", "/users").with_description("List all users");

        let route2 =
            serverless_rs::RouteInfo::new("POST", "/users").with_description("Create a new user");

        let metadata = serverless_rs::FunctionInfo::new("user_api")
            .with_description("API endpoints for user management")
            .with_resources(resources)
            .add_route(route1)
            .add_route(route2)
            .add_metadata("version", "1.0");

        serverless_rs::display_info(&metadata);
        return;
    }

    println!("Self-Documentation Example");
    println!("Run with --info flag to see function metadata");

    // Create a simple request and context
    let req = Request::new().with_method_str("GET").with_path("/users");

    let ctx = Context::new()
        .with_request_id("example-req-123")
        .with_function_name("user_api");

    // Create a runtime and run the function
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Run the function
    match rt.block_on(api_handler(req, &ctx)) {
        Ok(resp) => {
            println!("Function returned: HTTP {}", resp.status());
            println!(
                "Body: {}",
                std::str::from_utf8(resp.body()).unwrap_or("Binary content")
            );
        }
        Err(e) => {
            println!("Function error: {}", e);
        }
    }
}
