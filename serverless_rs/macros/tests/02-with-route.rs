//! Test for a serverless function with a #[route] attribute

use serverless_rs::{Context, Request, Response, Result};
use serverless_rs_macros::{route, serverless};

#[route(GET, "/hello", description = "Hello endpoint")]
#[serverless]
async fn hello_route(req: Request, ctx: &Context) -> Result<Response> {
    Ok(Response::text("Hello, world!"))
}

fn main() {
    // Check that function_info contains route information
    let info = hello_route::function_info();
    assert!(hello_route::has_route_info());

    assert_eq!(info.name, "hello_route");

    // Get route info and verify it
    let route = hello_route::route_info();
    assert_eq!(route.method, "GET");
    assert_eq!(route.path, "/hello");
    assert_eq!(route.description, Some("Hello endpoint".to_string()));
}
