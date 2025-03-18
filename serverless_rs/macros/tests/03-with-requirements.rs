//! Test for a serverless function with #[requirements] attribute

use serverless_rs::{Context, Request, Response, Result};
use serverless_rs_macros::{requirements, serverless};

#[requirements(
    recommend(memory = "128MB", timeout = "30s"),
    require(cpu = "1x"),
    platforms(aws, cloudflare),
    env(DATABASE_URL, API_KEY)
)]
#[serverless]
async fn handler_with_requirements(req: Request, ctx: &Context) -> Result<Response> {
    Ok(Response::text("Hello, world!"))
}

fn main() {
    // Check that the function has requirements
    assert!(handler_with_requirements::has_requirements());

    // Get requirements and verify them
    let reqs = handler_with_requirements::requirements();

    // Check that requirements were correctly processed
    assert!(reqs.platforms.contains(&"aws".to_string()));
    assert!(reqs.platforms.contains(&"cloudflare".to_string()));

    // Check environment variables
    assert!(reqs.environment.contains(&"DATABASE_URL".to_string()));
    assert!(reqs.environment.contains(&"API_KEY".to_string()));
}
