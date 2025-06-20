# Oxidized Framework Documentation

Welcome to the official documentation for Oxidized, a hyper-performant, modular, and asynchronous web framework for Rust. This document provides a comprehensive guide to understanding and utilizing the full capabilities of the framework.

## Table of Contents

1.  [**Introduction**](#1-introduction)
    -   [Philosophy](#philosophy)
    -   [Core Components](#core-components)
2.  [**Core Concepts**](#2-core-concepts)
    -   [Server](#server)
    -   [Router & Routing](#router--routing)
    -   [Handlers](#handlers)
    -   [Request](#request)
    -   [Response](#response)
    -   [Extractors](#extractors)
    -   [Middleware (Layers)](#middleware-layers)
    -   [Error Handling](#error-handling)
3.  [**Advanced Guides**](#3-advanced-guides)
    -   [State Management](#state-management)
    -   [Custom Extractors](#custom-extractors)
    -   [WebSocket Integration](#websocket-integration)
4.  [**Putting It All Together**](#4-putting-it-all-together)
    -   [A Complete JSON API Example](#a-complete-json-api-example)

---

## 1. Introduction

### Philosophy

Oxidized is engineered with two primary objectives: **maximum performance** and **developer ergonomics**. It leverages Rust's zero-cost abstractions and robust type system to provide a safe and efficient environment for building web services. The design is heavily inspired by the `Tower` ecosystem, promoting a modular and composable architecture through a `Service`-based design. This allows developers to build complex applications by assembling reusable components.

### Core Components

The framework is built upon a few key abstractions:

-   `Server`: The entry point that binds to a socket and handles incoming connections.
-   `Service`: A trait representing an asynchronous function that processes a request and returns a response. The entire application, including the router and middleware, is a `Service`.
-   `Router`: A `Service` that dispatches requests to different handler functions based on the request's URI and HTTP method.
-   `Layer`: A trait for creating middleware, which wraps a `Service` to add pre-processing or post-processing logic.

---

## 2. Core Concepts

### Server

The `Server` is responsible for managing the HTTP lifecycle. It listens for TCP connections, serves requests using your application logic, and can be configured for features like TLS.

```rust
use oxidized::{Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let router = Router::new(); // Your application logic
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create a server and run it
    Server::new(router, addr)
        .run()
        .await
        .unwrap();
}
```

#### TLS Configuration

To enable TLS, use the `.tls()` method, providing paths to your certificate and private key.

```rust
// In main()
Server::new(router, addr)
    .tls("path/to/cert.pem", "path/to/key.pem")
    .run()
    .await
    .unwrap();
```

### Router & Routing

The `Router` is used to define the endpoints of your application. It maps HTTP methods and URI paths to specific handler functions.

```rust
use oxidized::{Response, Result, Router};

async fn get_user() -> Result<Response> { /* ... */ Ok(Response::new("user_data")) }
async fn create_user() -> Result<Response> { /* ... */ Ok(Response::new("user_created")) }

let router = Router::new()
    .get("/users", get_user)
    .post("/users", create_user);
```

### Handlers

A handler is an `async` function that takes zero or more arguments (extractors) and returns a `Result<Response>`. This is where your application's business logic resides.

```rust
use oxidized::{Request, Response, Result};

// A simple handler with no arguments
async fn root() -> Result<Response> {
    Ok(Response::new("Hello, World!"))
}

// A handler that receives the request object
async fn echo(req: Request) -> Result<Response> {
    let body = req.into_body();
    Ok(Response::new(body))
}
```

### Request

The `Request` object encapsulates all information about an incoming HTTP request, including its method, URI, headers, and body. It is a lightweight wrapper around `hyper::Request`.

```rust
use oxidized::{Request, Response, Result};
use http::StatusCode;

async fn get_header(req: Request) -> Result<Response> {
    let user_agent = req.inner().headers()
        .get("user-agent")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    
    Ok(Response::new(format!("User-Agent: {}", user_agent)))
}
```

### Response

The `Response` object is used to construct the HTTP response sent back to the client. You can set the body, status code, and headers.

```rust
use oxidized::{Response, Result};
use http::{header, StatusCode};

async fn create_resource() -> Result<Response> {
    let mut response = Response::new("Resource Created");
    *response.status_mut() = StatusCode::CREATED;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/plain".parse().unwrap()
    );
    Ok(response)
}
```

### Extractors

Extractors are a powerful feature for deserializing parts of a request directly into your handler's arguments. This pattern promotes type safety and removes boilerplate parsing logic from your handlers.

To use an extractor, simply include its type in the handler's function signature.

#### `Json<T>` Extractor

The most common extractor is `Json<T>`, which deserializes a JSON request body into a typed struct.

```rust
use oxidized::{Json, Response, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

async fn create_user_handler(Json(payload): Json<CreateUser>) -> Result<Response> {
    // `payload` is now a type-safe `CreateUser` struct
    let response_body = format!("Created user: {}", payload.username);
    Ok(Response::new(response_body))
}
```

### Middleware (Layers)

Middleware allows you to insert logic into the request-processing lifecycle. In Oxidized, middleware is implemented using the `Layer` trait. Layers wrap a `Service`, allowing you to inspect or modify requests and responses.

#### Example: Logging Middleware

```rust
// See `src/middleware/log.rs` for the full implementation of `LogLayer`.
use oxidized::middleware::LogLayer;

// In your main function:
let router = Router::new()
    .get("/", root)
    .layer(LogLayer); // Apply the LogLayer to the router
```

A `Layer` is essentially a function that takes a `Service` and returns another `Service`. This composable design allows you to stack multiple middleware.

### Error Handling

Oxidized uses a standard `Result<T, E>`-based approach for error handling, with a custom `Error` enum. Your handlers should return a `Result<Response>`. If an `Err` is returned, the server will automatically map it to an appropriate HTTP error response.

---

## 3. Advanced Guides

### State Management

For applications that require shared state (e.g., a database connection pool), you can inject state using a custom middleware layer.

1.  **Define your state struct.**
2.  **Create a layer that adds the state to the request's extensions.**
3.  **Create a custom extractor to retrieve the state in your handler.**

```rust
// 1. Define state
#[derive(Clone)]
struct AppState {
    db_pool: Arc<DbPool>,
}

// 2. See `examples/state.rs` for a full implementation of
// the layer and extractor for state management.

// 3. Use in handler
async fn get_users_from_db(State(state): State<AppState>) -> Result<Response> {
    // Use `state.db_pool`
    Ok(Response::new("..."))
}
```
*Note: The `State` extractor is a common pattern that would need to be implemented as a custom extractor.*

### Custom Extractors

You can create your own extractors by implementing the `FromRequest` trait. This allows you to create reusable components for extracting any information from a request.

```rust
use async_trait::async_trait;
use oxidized::{Error, FromRequest, Request};

struct ApiKey(String);

#[async_trait]
impl FromRequest for ApiKey {
    type Rejection = Error;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Rejection> {
        if let Some(key) = req.inner().headers().get("x-api-key") {
            Ok(ApiKey(key.to_str().unwrap().to_string()))
        } else {
            Err(Error::Unauthorized)
        }
    }
}

// Handler using the custom extractor
async fn protected_route(api_key: ApiKey) -> Result<Response> {
    // `api_key` is now available
    Ok(Response::new("Access Granted"))
}
```

### WebSocket Integration

Oxidized provides first-class support for WebSockets. You can upgrade an HTTP connection to a WebSocket connection using the `.ws()` method on the `Router`.

```rust
use oxidized::{Router, ws::{Message, WebSocket}};

async fn websocket_handler(mut ws: WebSocket) {
    while let Some(Ok(msg)) = ws.recv().await {
        if let Message::Text(text) = msg {
            if ws.send(Message::Text(format!("Echo: {}", text))).await.is_err() {
                break;
            }
        }
    }
}

// In your router setup:
let router = Router::new().ws("/ws", websocket_handler);
```

---

## 4. Putting It All Together

### A Complete JSON API Example

This example demonstrates a simple in-memory JSON API with routing, state management, and middleware.

```rust
use oxidized::{
    extractor::Json,
    middleware::LogLayer,
    Response, Result, Router, Server,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

// In-memory database
type Db = Arc<RwLock<HashMap<u32, User>>>;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    username: String,
}

// State for our application
#[derive(Clone)]
struct AppState {
    db: Db,
}

// Custom State extractor (implementation required)
// struct State<T>(T);

async fn get_users(/* State(state): State<AppState> */) -> Result<Response> {
    // In a real implementation, you would use the state extractor.
    // let users: Vec<User> = state.db.read().unwrap().values().cloned().collect();
    // Ok(Response::from_json(&users))
    Ok(Response::new("[]")) // Placeholder
}

async fn create_user(
    Json(user): Json<User>,
    /* State(state): State<AppState> */
) -> Result<Response> {
    // state.db.write().unwrap().insert(user.id, user.clone());
    // Ok(Response::from_json(&user))
    Ok(Response::new("User created")) // Placeholder
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .get("/users", get_users)
        .post("/users", create_user)
        .layer(LogLayer);
        // .layer(StateLayer::new(state)); // Add state layer

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on {}", addr);
    Server::new(app, addr).run().await.unwrap();
}
```

This documentation provides the foundation for building robust applications with Oxidized. For more specific details, please refer to the source code and the `rustdoc` API documentation.
