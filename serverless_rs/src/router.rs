/*!
Router trait definition for serverless.rs.

This module defines the Router trait for HTTP routing in serverless functions.
*/

use async_trait::async_trait;
use http::Method;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    error::{Error, Result},
    Context, Handler, Request, Response,
};

/// A route handler function
pub type RouteHandler = Arc<dyn Handler>;

/// Router trait for handling HTTP routes in serverless functions
///
/// This trait defines the routing abstraction for HTTP-based serverless functions.
/// Implementations of this trait can be deployed to any supported serverless platform
/// that supports HTTP routing using the `#[serverless]` attribute.
///
/// # Examples
///
/// ```
/// use serverless_rs::{Router, Request, Response, Context};
/// use async_trait::async_trait;
/// use http::Method;
///
/// struct MyRouter;
///
/// #[async_trait]
/// impl Router for MyRouter {
///     async fn route(&self, req: Request, ctx: &Context) -> serverless_rs::Result<Response> {
///         match (req.method(), req.uri().map(|u| u.path())) {
///             (Some(&Method::GET), Some("/hello")) => {
///                 Ok(Response::text("Hello, world!"))
///             },
///             (Some(&Method::POST), Some("/users")) => {
///                 // Handle user creation
///                 Ok(Response::json(&serde_json::json!({"created": true})).unwrap())
///             },
///             _ => {
///                 Ok(Response::not_found())
///             }
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Router: Send + Sync + 'static {
    /// Route an HTTP request to the appropriate handler
    ///
    /// This method is called when the function receives an HTTP request. It should
    /// determine the appropriate handler for the request and return the response.
    async fn route(&self, req: Request, ctx: &Context) -> Result<Response>;
}

/// A builder for creating routers with route registration
#[derive(Default)]
#[allow(dead_code)]
pub struct RouterBuilder {
    routes: HashMap<(Method, String), RouteHandler>,
}

#[allow(dead_code)]
impl RouterBuilder {
    /// Create a new router builder
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Add a route to the router
    pub fn route<H>(mut self, method: Method, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler,
    {
        let path = path.into();
        self.routes.insert((method, path), Arc::new(handler));
        self
    }

    /// Add a GET route to the router
    pub fn get<H>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler,
    {
        self.route(Method::GET, path, handler)
    }

    /// Add a POST route to the router
    pub fn post<H>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler,
    {
        self.route(Method::POST, path, handler)
    }

    /// Add a PUT route to the router
    pub fn put<H>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Add a DELETE route to the router
    pub fn delete<H>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Build the router
    pub fn build(self) -> impl Router {
        BuildRouter {
            routes: self.routes,
        }
    }
}

/// Router implementation created by RouterBuilder
#[allow(dead_code)]
struct BuildRouter {
    routes: HashMap<(Method, String), RouteHandler>,
}

#[async_trait]
impl Router for BuildRouter {
    async fn route(&self, req: Request, ctx: &Context) -> Result<Response> {
        // Get the method and path from the request
        let method = req
            .method()
            .cloned()
            .ok_or_else(|| Error::http("Missing HTTP method"))?;
        let path = req
            .uri()
            .ok_or_else(|| Error::http("Missing request URI"))?
            .path()
            .to_string();

        // Find the handler for this route
        if let Some(handler) = self.routes.get(&(method.clone(), path.clone())) {
            handler.handle(req, ctx).await
        } else {
            // Return 404 if no handler is found
            Ok(Response::not_found())
        }
    }
}

/// Middleware support will be implemented in future versions
/// We'll keep the router simpler for now to pass compilation

#[cfg(test)]
mod tests {
    use super::*;

    struct HelloHandler;

    #[async_trait]
    impl Handler for HelloHandler {
        async fn handle(&self, _req: Request, _ctx: &Context) -> Result<Response> {
            Ok(Response::text("Hello, world!"))
        }
    }

    struct EchoHandler;

    #[async_trait]
    impl Handler for EchoHandler {
        async fn handle(&self, req: Request, _ctx: &Context) -> Result<Response> {
            let name = req
                .query_param("name")
                .cloned()
                .unwrap_or_else(|| "stranger".to_string());
            Ok(Response::text(format!("Hello, {}!", name)))
        }
    }

    #[tokio::test]
    async fn test_router_builder() {
        let router = RouterBuilder::new()
            .get("/hello", HelloHandler)
            .get("/echo", EchoHandler)
            .build();

        // Test hello route
        let req = Request::new()
            .with_method(Method::GET)
            .with_uri("/hello".parse().unwrap());
        let ctx = Context::new();
        let response = router.route(req, &ctx).await.unwrap();
        assert_eq!(
            std::str::from_utf8(response.body()).unwrap(),
            "Hello, world!"
        );

        // Test echo route
        let req = Request::new()
            .with_method(Method::GET)
            .with_uri("/echo".parse().unwrap())
            .with_query("name", "Test");
        let ctx = Context::new();
        let response = router.route(req, &ctx).await.unwrap();
        assert_eq!(
            std::str::from_utf8(response.body()).unwrap(),
            "Hello, Test!"
        );

        // Test not found
        let req = Request::new()
            .with_method(Method::GET)
            .with_uri("/unknown".parse().unwrap());
        let ctx = Context::new();
        let response = router.route(req, &ctx).await.unwrap();
        assert_eq!(response.status(), 404);
    }
}
