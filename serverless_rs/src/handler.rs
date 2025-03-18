/*!
Handler trait definition for serverless.rs.

This module defines the core Handler trait that serverless functions implement.
*/

use async_trait::async_trait;

use crate::{error::Result, Context, Request, Response};

/// Handler trait for implementing serverless functions
///
/// This trait defines the core abstraction for serverless functions. Implementations
/// of this trait can be deployed to any supported serverless platform using the
/// `#[serverless]` attribute.
///
/// # Examples
///
/// ```
/// use serverless_rs::{Handler, Request, Response, Context};
/// use async_trait::async_trait;
///
/// struct MyHandler;
///
/// #[async_trait]
/// impl Handler for MyHandler {
///     async fn handle(&self, req: Request, ctx: &Context) -> serverless_rs::Result<Response> {
///         // Log the incoming request
///         ctx.log("INFO", &format!("Received request: {:?}", req.method()));
///
///         // Return a simple response
///         Ok(Response::text("Hello from serverless.rs!"))
///     }
/// }
/// ```
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// Handle a serverless function invocation
    ///
    /// This method is called when the function is invoked. It receives the request
    /// and context for the invocation, and should return a response or an error.
    async fn handle(&self, req: Request, ctx: &Context) -> Result<Response>;
}

// Implement Handler for async functions
#[async_trait]
impl<F> Handler for F
where
    F: Send + Sync + 'static,
    F: Fn(Request, &Context) -> Result<Response> + Send + Sync,
{
    async fn handle(&self, req: Request, ctx: &Context) -> Result<Response> {
        (self)(req, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[tokio::test]
    async fn test_function_handler() {
        // Define a simple handler function
        fn my_handler(req: Request, _ctx: &Context) -> Result<Response> {
            if req.path_param("id").is_some() {
                Ok(Response::text("Found item"))
            } else {
                Err(Error::http("Missing id parameter"))
            }
        }

        // Create a wrapper to make it a Handler
        struct HandlerWrapper<F: Fn(Request, &Context) -> Result<Response> + Sync + Send> {
            handler: F,
        }

        #[async_trait]
        impl<F> Handler for HandlerWrapper<F>
        where
            F: Fn(Request, &Context) -> Result<Response> + Sync + Send + 'static,
        {
            async fn handle(&self, req: Request, ctx: &Context) -> Result<Response> {
                (self.handler)(req, ctx)
            }
        }

        let handler = HandlerWrapper {
            handler: my_handler,
        };

        // Test successful case
        let req = Request::new().with_path_param("id", "123");
        let ctx = Context::new();
        let response = handler.handle(req, &ctx).await.unwrap();
        assert_eq!(std::str::from_utf8(response.body()).unwrap(), "Found item");

        // Test error case
        let req = Request::new();
        let ctx = Context::new();
        let result = handler.handle(req, &ctx).await;
        assert!(result.is_err());
    }

    struct TestHandler;

    #[async_trait]
    impl Handler for TestHandler {
        async fn handle(&self, req: Request, _ctx: &Context) -> Result<Response> {
            if let Some(name) = req.query_param("name") {
                Ok(Response::text(format!("Hello, {}!", name)))
            } else {
                Ok(Response::text("Hello, world!"))
            }
        }
    }

    #[tokio::test]
    async fn test_struct_handler() {
        let handler = TestHandler;

        // Test with query parameter
        let req = Request::new().with_query("name", "Test");
        let ctx = Context::new();
        let response = handler.handle(req, &ctx).await.unwrap();
        assert_eq!(
            std::str::from_utf8(response.body()).unwrap(),
            "Hello, Test!"
        );

        // Test without query parameter
        let req = Request::new();
        let ctx = Context::new();
        let response = handler.handle(req, &ctx).await.unwrap();
        assert_eq!(
            std::str::from_utf8(response.body()).unwrap(),
            "Hello, world!"
        );
    }
}
