# Lecture 03: HTTP Servers with Axum

## Introduction

HTTP is the foundation of web applications and REST APIs. Axum is a modern, ergonomic web framework built on top of Tokio that makes building HTTP services a pleasure.

**Duration:** 90 minutes

## Why Axum?

**Axum advantages:**
- Built on `tower` - industry-standard middleware
- Type-safe extractors - compile-time request validation
- Great error messages - Rust's type system helps
- Excellent performance - minimal overhead
- Seamless integration with Tokio ecosystem

**Alternatives:**
- `actix-web` - slightly faster, more mature
- `warp` - filter-based, more functional style
- `rocket` - batteries-included, slightly less flexible

## Your First HTTP Server

```rust
use axum::{
    Router,
    routing::get,
    response::IntoResponse,
};

#[tokio::main]
async fn main() {
    // Build our application with a single route
    let app = Router::new()
        .route("/", get(handler));

    // Run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
```

**Testing:**
```bash
curl http://localhost:3000
# Output: Hello, World!
```

## Routing

### Basic Routes

```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
};

let app = Router::new()
    .route("/", get(index))
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
    .route("/health", get(health_check));

async fn index() -> &'static str {
    "Welcome to the API"
}

async fn health_check() -> &'static str {
    "OK"
}
```

### Path Parameters

```rust
use axum::{
    extract::Path,
    http::StatusCode,
};

// GET /users/123
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User ID: {}", id)
}

// Multiple parameters: GET /users/123/posts/456
async fn get_user_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
    format!("User {} Post {}", user_id, post_id)
}

// Using a struct for clarity
#[derive(Debug, serde::Deserialize)]
struct UserPostParams {
    user_id: u64,
    post_id: u64,
}

async fn get_user_post_v2(Path(params): Path<UserPostParams>) -> String {
    format!("User {} Post {}", params.user_id, params.post_id)
}
```

### Query Parameters

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

// GET /users?page=2&per_page=20
async fn list_users(Query(params): Query<Pagination>) -> String {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    format!("Page {}, {} items per page", page, per_page)
}
```

## Request and Response Handling

### JSON Requests and Responses

```rust
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<User>) {
    // In real app, save to database
    let user = User {
        id: 1,
        name: payload.name,
        email: payload.email,
    };

    (StatusCode::CREATED, Json(user))
}
```

**Testing:**
```bash
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "email": "alice@example.com"}'
```

### Custom Response Types

```rust
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

// Simple text response
async fn handler1() -> &'static str {
    "Plain text response"
}

// With status code
async fn handler2() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created")
}

// JSON response
async fn handler3() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    })
}

// Custom response with headers
async fn handler4() -> Response {
    (
        StatusCode::OK,
        [("X-Custom-Header", "value")],
        "Response with custom header"
    ).into_response()
}
```

## Error Handling

### Using Result

```rust
use axum::{
    http::StatusCode,
    response::IntoResponse,
};

async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, StatusCode> {
    match find_user_in_db(id).await {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
```

### Custom Error Type

```rust
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::Serialize;

#[derive(Debug)]
enum AppError {
    NotFound,
    DatabaseError(String),
    ValidationError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, &msg),
        };

        let body = Json(ErrorResponse {
            error: status.to_string(),
            message: message.to_string(),
        });

        (status, body).into_response()
    }
}

// Usage
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    let user = find_user(id).await.ok_or(AppError::NotFound)?;
    Ok(Json(user))
}
```

## State Management

### Shared State with Arc

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router,
    extract::State,
    routing::get,
};

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<Vec<User>>>,
    counter: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        users: Arc::new(RwLock::new(Vec::new())),
        counter: Arc::new(RwLock::new(0)),
    };

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/stats", get(get_stats))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = state.users.read().await;
    Json(users.clone())
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<User>) {
    let mut counter = state.counter.write().await;
    *counter += 1;

    let user = User {
        id: *counter,
        name: payload.name,
        email: payload.email,
    };

    let mut users = state.users.write().await;
    users.push(user.clone());

    (StatusCode::CREATED, Json(user))
}

async fn get_stats(State(state): State<AppState>) -> String {
    let users = state.users.read().await;
    let counter = state.counter.read().await;
    format!("Users: {}, Total created: {}", users.len(), counter)
}
```

### Database Connection Pool

```rust
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let db = PgPool::connect(&database_url).await?;

    let state = AppState { db };

    let app = Router::new()
        .route("/users", get(list_users))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(users))
}
```

## Middleware

### Logging Middleware

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http());
```

### CORS Middleware

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .route("/api/users", get(list_users))
    .layer(cors);
```

### Custom Authentication Middleware

```rust
use axum::{
    middleware::{self, Next},
    http::{Request, StatusCode},
    response::Response,
};

async fn auth_middleware<B>(
    headers: axum::http::HeaderMap,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    // Validate token (simplified)
    if token != "valid-token" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add user ID to extensions
    request.extensions_mut().insert(UserId(123));

    Ok(next.run(request).await)
}

// Apply middleware to specific routes
let app = Router::new()
    .route("/public", get(public_handler))
    .route("/protected", get(protected_handler))
    .layer(middleware::from_fn(auth_middleware));

// Access user ID in handler
async fn protected_handler(
    Extension(user_id): Extension<UserId>,
) -> String {
    format!("Hello, user {}!", user_id.0)
}

#[derive(Clone)]
struct UserId(u64);
```

## Advanced Patterns

### Nested Routers

```rust
let users_router = Router::new()
    .route("/", get(list_users).post(create_user))
    .route("/:id", get(get_user).put(update_user).delete(delete_user));

let posts_router = Router::new()
    .route("/", get(list_posts).post(create_post))
    .route("/:id", get(get_post));

let app = Router::new()
    .nest("/api/users", users_router)
    .nest("/api/posts", posts_router)
    .route("/health", get(health_check));

// Routes:
// GET  /api/users
// POST /api/users
// GET  /api/users/123
// GET  /api/posts
// etc.
```

### File Uploads

```rust
use axum::{
    extract::Multipart,
    response::Html,
};

async fn upload_file(mut multipart: Multipart) -> Result<String, AppError> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Field: {}, Size: {} bytes", name, data.len());

        // Save file
        tokio::fs::write(format!("uploads/{}", name), data).await.unwrap();
    }

    Ok("File uploaded successfully".to_string())
}

async fn upload_form() -> Html<&'static str> {
    Html(r#"
        <!doctype html>
        <html>
            <body>
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <input type="file" name="file">
                    <input type="submit" value="Upload">
                </form>
            </body>
        </html>
    "#)
}
```

### Server-Sent Events (SSE)

```rust
use axum::{
    response::sse::{Event, Sse},
    response::IntoResponse,
};
use futures::stream::{self, Stream};
use std::time::Duration;
use std::convert::Infallible;

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| {
        Event::default()
            .data(format!("Current time: {}", chrono::Utc::now()))
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream)
}
```

## WebSocket Support

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        };

        if let axum::extract::ws::Message::Text(text) = msg {
            let response = format!("Server received: {}", text);
            if socket.send(axum::extract::ws::Message::Text(response)).await.is_err() {
                break;
            }
        }
    }
}

// In router:
let app = Router::new()
    .route("/ws", get(ws_handler));
```

## Testing HTTP Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_user() {
        let app = Router::new().route("/users/:id", get(get_user));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users/123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_user() {
        let app = Router::new().route("/users", post(create_user));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Alice","email":"alice@example.com"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
```

## Complete CRUD Example

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<Vec<User>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        users: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = state.users.read().await;
    Json(users.clone())
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
    };

    state.users.write().await.push(user.clone());

    (StatusCode::CREATED, Json(user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.read().await;
    users
        .iter()
        .find(|u| u.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let mut users = state.users.write().await;

    let user = users
        .iter_mut()
        .find(|u| u.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    user.name = payload.name;
    user.email = payload.email;

    Ok(Json(user.clone()))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let mut users = state.users.write().await;

    if let Some(pos) = users.iter().position(|u| u.id == id) {
        users.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
```

## Exercises

1. **Blog API**: Build a CRUD API for blog posts with title, content, author
2. **Authentication**: Add JWT authentication to protect certain routes
3. **File Upload**: Implement image upload with validation and storage
4. **Real-time Chat**: Build a chat API using WebSockets

## Key Takeaways

1. **Axum uses extractors** - Path, Query, Json, State
2. **Type-safe routing** - compile-time guarantees
3. **Middleware via tower** - composable, reusable
4. **State with Arc<RwLock>** - shared across handlers
5. **IntoResponse trait** - flexible response types

## Next Lecture

We'll cover channels and concurrent patterns:
- mpsc, broadcast, watch channels
- Select and join macros
- Cancellation and timeouts
- Actor pattern

## Resources

- [Axum Documentation](https://docs.rs/axum/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [Tower Middleware](https://docs.rs/tower/)
