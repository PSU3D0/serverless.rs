# AWS Lambda Integration Plan for serverless.rs

## 1. Overview and Approach

The AWS Lambda adapter for serverless.rs will provide a minimal, focused translation layer between the platform-agnostic serverless.rs abstractions and AWS Lambda's runtime model. This document outlines our technical approach that maintains the project's core ethos: "one macro, multiple clouds" with strict separation of concerns.

## 2. Core Design Principles

- **Minimal Coupling**: Avoid unnecessary dependencies on AWS-specific services or SDKs beyond what's required for Lambda integration
- **Developer Freedom**: Provide references to AWS services but let developers choose how to interact with them
- **Clear Boundaries**: Separate framework responsibilities (runtime integration) from developer/IaC responsibilities (resource provisioning)
- **Feature Isolation**: Ensure AWS-specific code is isolated behind feature flags
- **Performance First**: Optimize for minimal overhead and cold start times

## 3. Technical Approach

### 3.1 Core Lambda Handler Adapter

- **Entry Point Generation**: Generate AWS Lambda-compatible entry points at compile time via macros
- **Cold Start Optimization**: Minimize initialization overhead with compile-time code generation
- **Error Handling**: Map serverless.rs errors to Lambda response formats
- **Context Mapping**: Provide access to Lambda runtime context via our Context abstraction

### 3.2 Event Source Handling

- **API Gateway Integration**: 
  - Map API Gateway events to our HTTP abstractions
  - Preserve access to raw API Gateway event details when needed
  - Support path parameters and query strings

- **Event Deserialization**: 
  - Provide type-safe parsing for common Lambda event types
  - Maintain raw event access for developer flexibility
  - No tight coupling with specific AWS service SDKs

- **Direct Invocation**:
  - Support basic Lambda function invocation patterns
  - Handle standard request/response serialization

### 3.3 Lambda Runtime Access

- **Context References**: Expose references to Lambda context objects
- **Service References**: Provide access to Lambda-injected service connections (rather than creating abstractions)
- **Credential Access**: Pass through Lambda's credential providers

## 4. AWS Services Approach

- **Primary Focus**: AWS Lambda runtime integration only
- **Secondary Services**: Expose references only - no custom abstractions
  - Provide access to CloudWatch, XRay contexts when available
  - Do not create framework-specific wrappers for AWS services
  - Allow developers to use standard AWS SDKs directly

## 5. IaC Compatibility

- **Clear Separation**: Framework handles runtime integration; IaC tools handle provisioning
- **Resource Requirements**: Expose function requirements via the `--info` flag 
- **Terraform Recommendations**: Generate resource recommendations without tightly coupling to specific Terraform patterns

## 6. Implementation Strategy

### 6.1 Feature Flags

- **Simple Approach**: Single `aws` feature flag for all Lambda functionality
- **HTTP Integration**: Integrate API Gateway under the same `aws` flag 
- **No Service-Specific Flags**: Avoid fine-grained feature flags for AWS services

### 6.2 Implementation Phases

1. **Phase 1: Basic Lambda Handler**
   - Implement minimal Lambda function entry point adapter
   - Map basic Lambda context to serverless.rs Context
   - Support direct invocation with JSON serialization

2. **Phase 2: API Gateway Integration**
   - Integrate API Gateway HTTP events with our Router
   - Map API Gateway context and event details
   - Support path parameters and query strings

3. **Phase 3: Event Type Support**
   - Add parsing for common AWS event types
   - Ensure raw event data is accessible
   - Maintain minimal abstractions

### 6.3 Testing Strategy

- Unit tests for event parsing and mapping
- Integration tests using LocalStack
- Performance benchmarking against raw Lambda implementations

## 7. Developer Experience

### 7.1 Minimal Approach Example

```rust
#[serverless]
async fn handle_event(req: Request, ctx: Context) -> Result<Response, Error> {
    // Platform-agnostic code
    let user_id = req.query_param("user_id")?;
    
    // If developer needs AWS-specific functionality:
    #[cfg(feature = "aws")]
    {
        // Developer directly uses AWS SDK with references from context
        let dynamo_client = aws_sdk_dynamodb::Client::new(&ctx.aws_credentials());
        let result = dynamo_client.get_item()
            .table_name("Users")
            .key("id", AttributeValue::S(user_id))
            .send()
            .await?;
        
        // Access Lambda/API Gateway specific details when needed
        if let Some(api_gateway_event) = ctx.aws_raw_event() {
            // Work with API Gateway specific properties
        }
        
        // Access Lambda context properties
        if let Some(lambda_context) = ctx.aws_lambda_context() {
            let request_id = lambda_context.request_id();
        }
    }
    
    // Return platform-agnostic response
    Ok(Response::json(result))
}
```

## 8. Performance Considerations

- Minimize binary size through careful dependency management
- Use compile-time code generation over runtime reflection
- Implement efficient serialization/deserialization
- Provide direct access to AWS types to avoid conversion overhead

## 9. Open Questions

1. What Lambda context properties should we expose directly vs. through raw access?
2. How should we handle API Gateway authorizer context information?
3. What's the most ergonomic way to provide access to event-specific data?
4. How do we balance type safety with flexibility for custom event types?



## Examples of using Lambda from aws-lambda-rust-runtime


```basic-lambda.rs
// This example requires the following input to succeed:
// { "command": "do something" }

use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// This is also a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // extract some useful info from the request
    let command = event.payload.command;

    // prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {} executed.", command),
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use crate::{my_handler, Request};
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn response_is_good_for_simple_input() {
        let id = "ID";

        let mut context = Context::default();
        context.request_id = id.to_string();

        let payload = Request {
            command: "X".to_string(),
        };
        let event = LambdaEvent { payload, context };

        let result = my_handler(event).await.unwrap();

        assert_eq!(result.msg, "Command X executed.");
        assert_eq!(result.req_id, id.to_string());
    }
}
```



```basic-error-thiserror.rs
use lambda_runtime::{service_fn, Diagnostic, Error, LambdaEvent};
use serde::Deserialize;
use thiserror;

#[derive(Deserialize)]
struct Request {}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("transient database error: {0}")]
    DatabaseError(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

impl From<ExecutionError> for Diagnostic {
    fn from(value: ExecutionError) -> Diagnostic {
        let (error_type, error_message) = match value {
            ExecutionError::DatabaseError(err) => ("Retryable", err.to_string()),
            ExecutionError::Unexpected(err) => ("NonRetryable", err.to_string()),
        };
        Diagnostic {
            error_type: error_type.into(),
            error_message: error_message.into(),
        }
    }
}

/// This is the main body for the Lambda function
async fn function_handler(_event: LambdaEvent<Request>) -> Result<(), ExecutionError> {
    Err(ExecutionError::Unexpected("ooops".to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(function_handler)).await
}
```


```basic-streaming-response.rs
use lambda_runtime::{
    service_fn,
    streaming::{channel, Body, Response},
    tracing, Error, LambdaEvent,
};
use serde_json::Value;
use std::{thread, time::Duration};

async fn func(_event: LambdaEvent<Value>) -> Result<Response<Body>, Error> {
    let messages = vec!["Hello", "world", "from", "Lambda!"];

    let (mut tx, rx) = channel();

    tokio::spawn(async move {
        for message in messages.iter() {
            tx.send_data((message.to_string() + "\n").into()).await.unwrap();
            thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(Response::from(rx))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    lambda_runtime::run(service_fn(func)).await?;
    Ok(())
}
```


```http-basic-lambda.rs
use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code examples in the Runtime repository:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
```

```http-axum-diesel.rs
use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use bb8::Pool;
use diesel::prelude::*;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl};
use lambda_http::{http::StatusCode, run, tracing, Error};
use serde::{Deserialize, Serialize};

table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        content -> Text,
        published -> Bool,
    }
}

#[derive(Default, Queryable, Selectable, Serialize)]
struct Post {
    id: i32,
    title: String,
    content: String,
    published: bool,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = posts)]
struct NewPost {
    title: String,
    content: String,
    published: bool,
}

type AsyncPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type ServerError = (StatusCode, String);

async fn create_post(State(pool): State<AsyncPool>, Json(post): Json<NewPost>) -> Result<Json<Post>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let post = diesel::insert_into(posts::table)
        .values(post)
        .returning(Post::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(post))
}

async fn list_posts(State(pool): State<AsyncPool>) -> Result<Json<Vec<Post>>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let posts = posts::table
        .filter(posts::dsl::published.eq(true))
        .load(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(posts))
}

async fn get_post(State(pool): State<AsyncPool>, Path(post_id): Path<i32>) -> Result<Json<Post>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let post = posts::table
        .find(post_id)
        .first(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(post))
}

async fn delete_post(State(pool): State<AsyncPool>, Path(post_id): Path<i32>) -> Result<(), ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    diesel::delete(posts::table.find(post_id))
        .execute(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(())
}

fn internal_server_error<E: std::error::Error>(err: E) -> ServerError {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    // Set up the database connection
    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL environment variable");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    let connection = Pool::builder()
        .build(config)
        .await
        .expect("unable to establish the database connection");

    // Set up the API routes
    let posts_api = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(get_post).delete(delete_post));
    let app = Router::new().nest("/posts", posts_api).with_state(connection);

    run(app).await
}
```

```basic-cognitio-post-confirmation.rs
use aws_config::BehaviorVersion;
use aws_lambda_events::event::cognito::CognitoEventUserPoolsPostConfirmation;
use aws_sdk_ses::{
    types::{Body, Content, Destination, Message},
    Client,
};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

const SOURCE_EMAIL: &str = "<source_email>";

async fn function_handler(
    client: &aws_sdk_ses::Client,
    event: LambdaEvent<CognitoEventUserPoolsPostConfirmation>,
) -> Result<CognitoEventUserPoolsPostConfirmation, Error> {
    let payload = event.payload;

    if let Some(email) = payload.request.user_attributes.get("email") {
        let body = if let Some(name) = payload.request.user_attributes.get("name") {
            format!("Welcome {name}, you have been confirmed.")
        } else {
            "Welcome, you have been confirmed.".to_string()
        };
        send_post_confirmation_email(client, email, "Cognito Identity Provider registration completed", &body).await?;
    }

    // Cognito always expect a response with the same shape as
    // the event when it handles Post Confirmation triggers.
    Ok(payload)
}

async fn send_post_confirmation_email(client: &Client, email: &str, subject: &str, body: &str) -> Result<(), Error> {
    let destination = Destination::builder().to_addresses(email).build();
    let subject = Content::builder().data(subject).build()?;
    let body = Content::builder().data(body).build()?;

    let message = Message::builder()
        .body(Body::builder().text(body).build())
        .subject(subject)
        .build();

    client
        .send_email()
        .source(SOURCE_EMAIL)
        .destination(destination)
        .message(message)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    run(service_fn(|event| function_handler(&client, event))).await
}
```