/*!
Simple example of using the serverless.rs framework.
*/

use serverless_rs::{Context, Request, Response, Result};

// Simple serverless function without using macros
async fn hello_world(_req: Request, ctx: &Context) -> Result<Response> {
    ctx.log("INFO", "Hello function called");
    Ok(Response::text("Hello, world!"))
}

// No need for a main function, the serverless macro handles that
fn main() {
    println!("Simple serverless.rs example");

    // Create a simple request and context
    let req = Request::new();
    let ctx = Context::new().with_request_id("example-req-123");

    // Invoke the function
    let future = hello_world(req, &ctx);

    // Use a simple runtime to run the future
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Run the future and print the result
    match rt.block_on(future) {
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
