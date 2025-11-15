# Module 12: Rust + TypeScript Interop

**Build TypeScript/JavaScript Services with Rust Backends**

## Overview

Learn to integrate Rust with TypeScript ecosystems:
- HTTP APIs with axum/actix-web
- WebAssembly (WASM) for browsers and Node.js
- TypeScript type generation
- Real-time communication (WebSocket, SSE)
- Microservices architecture
- GraphQL APIs

**Duration**: 2 weeks (20-25 hours)

## What You'll Build

### Approach 1: HTTP API Backend

```typescript
// TypeScript client
import { ApiClient } from './generated/api-client';

const client = new ApiClient('http://localhost:8000');

// Call Rust API
const user = await client.users.create({
  name: 'Alice',
  email: 'alice@example.com',
});

const users = await client.users.list({ limit: 10 });
console.log(users);
```

### Approach 2: WebAssembly Module

```typescript
// TypeScript using WASM module
import * as wasm from './pkg/my_wasm_lib';

// Call Rust functions from JavaScript
const result = wasm.compute_heavy_task(data);

// Use Rust classes in TypeScript
const processor = new wasm.DataProcessor();
processor.add_data([1, 2, 3, 4, 5]);
const stats = processor.get_statistics();
```

## Architecture Options

### Option 1: HTTP API Service

```
┌──────────────────────────────────┐
│   TypeScript/React Frontend      │
│   (or Node.js Backend)           │
└────────────┬─────────────────────┘
             │ HTTP/REST or GraphQL
             │
┌────────────▼─────────────────────┐
│      Rust HTTP Server            │
│   (axum, actix-web, or warp)     │
├──────────────────────────────────┤
│   - JSON API                     │
│   - WebSocket support            │
│   - Authentication               │
│   - Database access              │
└──────────────────────────────────┘
```

### Option 2: WebAssembly Module

```
┌──────────────────────────────────┐
│     Browser or Node.js           │
│   TypeScript Application         │
└────────────┬─────────────────────┘
             │ import from WASM
             │
┌────────────▼─────────────────────┐
│      Rust WASM Module            │
│   (wasm-bindgen, wasm-pack)      │
├──────────────────────────────────┤
│   - Fast computations            │
│   - CPU-intensive tasks          │
│   - Shared between browser/Node  │
└──────────────────────────────────┘
```

## Part 1: HTTP API Backend

### 1. Axum REST API

```rust
// src/main.rs
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
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
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct ListUsersQuery {
    limit: Option<usize>,
    offset: Option<usize>,
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
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/:id", get(get_user).delete(delete_user))
        .route("/health", get(health_check))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Server running on http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now(),
    };

    state.users.write().await.push(user.clone());

    Ok(Json(user))
}

async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Json<Vec<User>> {
    let users = state.users.read().await;

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10);

    let results: Vec<User> = users
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();

    Json(results)
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

### 2. TypeScript Client (Generated)

```typescript
// generated/api-client.ts
export interface User {
  id: string;
  name: string;
  email: string;
  created_at: string;
}

export interface CreateUserRequest {
  name: string;
  email: string;
}

export class ApiClient {
  constructor(private baseUrl: string) {}

  users = {
    create: async (data: CreateUserRequest): Promise<User> => {
      const response = await fetch(`${this.baseUrl}/api/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return response.json();
    },

    list: async (params?: { limit?: number; offset?: number }): Promise<User[]> => {
      const query = new URLSearchParams();
      if (params?.limit) query.set('limit', params.limit.toString());
      if (params?.offset) query.set('offset', params.offset.toString());

      const response = await fetch(`${this.baseUrl}/api/users?${query}`);
      return response.json();
    },

    get: async (id: string): Promise<User> => {
      const response = await fetch(`${this.baseUrl}/api/users/${id}`);
      if (!response.ok) {
        throw new Error('User not found');
      }
      return response.json();
    },

    delete: async (id: string): Promise<void> => {
      await fetch(`${this.baseUrl}/api/users/${id}`, { method: 'DELETE' });
    },
  };
}
```

### 3. OpenAPI/Swagger Integration

```rust
// Generate OpenAPI spec automatically
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        list_users,
        get_user,
        delete_user,
    ),
    components(schemas(User, CreateUserRequest))
)]
struct ApiDoc;

// In main():
let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    .route("/api/users", post(create_user).get(list_users))
    // ... rest of routes
```

Generate TypeScript types:
```bash
# Use openapi-typescript
npx openapi-typescript http://localhost:8000/api-docs/openapi.json --output ./generated/api-types.ts
```

## Part 2: WebAssembly Integration

### 1. Rust WASM Module

```rust
// lib.rs
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// A simple greeting function
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// CPU-intensive calculation
#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// Process array data
#[wasm_bindgen]
pub fn sum_array(numbers: &[f64]) -> f64 {
    numbers.iter().sum()
}

/// Struct exposed to JavaScript
#[wasm_bindgen]
pub struct DataProcessor {
    data: Vec<f64>,
}

#[wasm_bindgen]
impl DataProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DataProcessor {
        DataProcessor { data: Vec::new() }
    }

    pub fn add_data(&mut self, values: Vec<f64>) {
        self.data.extend(values);
    }

    pub fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

// For complex objects, use serde-wasm-bindgen
#[derive(Serialize, Deserialize)]
pub struct UserData {
    name: String,
    age: u32,
    email: String,
}

#[wasm_bindgen]
pub fn process_user(user_js: JsValue) -> Result<JsValue, JsValue> {
    let user: UserData = serde_wasm_bindgen::from_value(user_js)?;

    // Process user...
    let result = format!("{} is {} years old", user.name, user.age);

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

**Cargo.toml for WASM:**
```toml
[package]
name = "my-wasm-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### 2. Building WASM

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for bundlers (webpack, vite, etc.)
wasm-pack build --target bundler

# Build for Node.js
wasm-pack build --target nodejs

# Build for web (no bundler)
wasm-pack build --target web
```

### 3. TypeScript Usage

```typescript
// Using in TypeScript (with bundler)
import init, { greet, fibonacci, DataProcessor } from './pkg/my_wasm_lib';

async function main() {
  // Initialize WASM module
  await init();

  // Call functions
  console.log(greet('TypeScript'));
  console.log('fib(10):', fibonacci(10));

  // Use classes
  const processor = new DataProcessor();
  processor.add_data([1, 2, 3, 4, 5]);
  console.log('Mean:', processor.mean());
}

main();
```

### 4. TypeScript Type Definitions

wasm-pack automatically generates TypeScript definitions:

```typescript
// pkg/my_wasm_lib.d.ts (auto-generated)
export function greet(name: string): string;
export function fibonacci(n: number): number;
export function sum_array(numbers: Float64Array): number;

export class DataProcessor {
  constructor();
  add_data(values: Float64Array): void;
  mean(): number;
  clear(): void;
}
```

## Part 3: WebSocket Real-Time Communication

### Rust WebSocket Server

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
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
            // Echo the message back
            let response = format!("Server received: {}", text);
            if socket.send(axum::extract::ws::Message::Text(response)).await.is_err() {
                break;
            }
        }
    }
}
```

### TypeScript WebSocket Client

```typescript
class WebSocketClient {
  private ws: WebSocket;

  constructor(url: string) {
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('Connected to WebSocket');
    };

    this.ws.onmessage = (event) => {
      console.log('Received:', event.data);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket closed');
    };
  }

  send(message: string) {
    this.ws.send(message);
  }

  close() {
    this.ws.close();
  }
}

// Usage
const client = new WebSocketClient('ws://localhost:8000/ws');
client.send('Hello, Rust!');
```

## Part 4: GraphQL API

```rust
use async_graphql::{
    Context, Object, Schema, EmptySubscription, Result,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

struct User {
    id: String,
    name: String,
    email: String,
}

#[Object]
impl User {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn email(&self) -> &str {
        &self.email
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        // Fetch users from database/state
        Ok(vec![
            User {
                id: "1".to_string(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }
        ])
    }

    async fn user(&self, id: String) -> Result<Option<User>> {
        // Fetch single user
        Ok(None)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, name: String, email: String) -> Result<User> {
        Ok(User {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
        })
    }
}

type MySchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(
    schema: axum::extract::Extension<MySchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
```

TypeScript GraphQL Client:
```typescript
import { GraphQLClient, gql } from 'graphql-request';

const client = new GraphQLClient('http://localhost:8000/graphql');

const query = gql`
  query GetUsers {
    users {
      id
      name
      email
    }
  }
`;

const data = await client.request(query);
console.log(data);
```

## Implementation Roadmap

### Phase 1: REST API Backend (Days 1-4)
- Set up axum server
- Implement CRUD endpoints
- Add error handling
- CORS configuration

**Success criteria:**
- All REST endpoints work
- Proper error responses
- Can curl all endpoints

### Phase 2: TypeScript Client (Days 5-7)
- Generate TypeScript types
- Build client library
- Add type safety
- Error handling

**Success criteria:**
- Type-safe API calls
- Good developer experience
- Comprehensive error handling

### Phase 3: WebAssembly (Days 8-11)
- Set up WASM project
- Expose functions to JS
- Build and package
- Integration tests

**Success criteria:**
- WASM module loads in browser
- Functions callable from TypeScript
- Good performance

### Phase 4: Real-Time Features (Days 12-15)
- WebSocket support
- Server-Sent Events
- Real-time notifications
- Connection management

**Success criteria:**
- Bi-directional communication
- Handles disconnects gracefully
- Low latency

### Phase 5: Production Ready (Days 16-20)
- Authentication/Authorization
- Rate limiting
- Logging and monitoring
- Deployment guide

## Performance Comparison

### REST API Benchmark

```typescript
// benchmark.ts
async function benchmarkApi() {
  const iterations = 10000;

  const start = Date.now();

  const promises = [];
  for (let i = 0; i < iterations; i++) {
    promises.push(
      fetch('http://localhost:8000/api/users', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: `User ${i}`, email: `user${i}@example.com` }),
      })
    );
  }

  await Promise.all(promises);

  const elapsed = Date.now() - start;
  console.log(`${iterations} requests in ${elapsed}ms`);
  console.log(`${(iterations / elapsed * 1000).toFixed(0)} req/sec`);
}
```

### WASM vs JavaScript

```typescript
// Compare WASM vs JS performance
import * as wasm from './pkg/my_wasm_lib';

function fibonacciJS(n: number): number {
  if (n <= 1) return n;
  return fibonacciJS(n - 1) + fibonacciJS(n - 2);
}

console.time('JavaScript fib(40)');
fibonacciJS(40);
console.timeEnd('JavaScript fib(40)');

console.time('WASM fib(40)');
wasm.fibonacci(40);
console.timeEnd('WASM fib(40)');
```

## Testing

### Rust Integration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_user() {
        let app = create_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/users")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Alice","email":"alice@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### TypeScript Tests

```typescript
// api-client.test.ts
import { ApiClient } from './api-client';

describe('ApiClient', () => {
  const client = new ApiClient('http://localhost:8000');

  test('create user', async () => {
    const user = await client.users.create({
      name: 'Alice',
      email: 'alice@example.com',
    });

    expect(user.name).toBe('Alice');
    expect(user.id).toBeDefined();
  });

  test('list users', async () => {
    const users = await client.users.list({ limit: 10 });
    expect(Array.isArray(users)).toBe(true);
  });
});
```

## Success Criteria

- ✅ Build REST APIs with Rust
- ✅ Type-safe TypeScript client
- ✅ WebAssembly modules for browsers
- ✅ WebSocket real-time communication
- ✅ GraphQL API (optional)
- ✅ Production-ready deployment
- ✅ Comprehensive testing
- ✅ Good documentation

## Common Patterns

### 1. Error Handling

```rust
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}
```

### 2. Authentication Middleware

```rust
async fn auth_middleware(
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token...

    Ok(next.run(req).await)
}
```

### 3. Rate Limiting

```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(50)
        .finish()
        .unwrap(),
);

let app = Router::new()
    .route("/api/users", get(list_users))
    .layer(GovernorLayer { config: governor_conf });
```

## Resources

**HTTP Frameworks:**
- [Axum Documentation](https://docs.rs/axum/)
- [Actix-web](https://actix.rs/)
- [Warp](https://docs.rs/warp/)

**WebAssembly:**
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)

**GraphQL:**
- [async-graphql](https://async-graphql.github.io/async-graphql/en/index.html)

**Real Projects:**
- [SWC](https://swc.rs/) - Fast TypeScript/JavaScript compiler
- [Deno](https://deno.land/) - JavaScript runtime with Rust core
- [Turbopack](https://turbo.build/pack) - Rust-based bundler

## Conclusion

This module completes your journey from Rust beginner to systems expert! You've learned:

- **Modules 1-2**: Core Rust and async programming
- **Modules 3-7**: Building real distributed systems
- **Modules 8-9**: Advanced projects (database, compiler)
- **Module 10**: Capstone trading system
- **Modules 11-12**: Integration with Python and TypeScript

You're now equipped to build production systems in Rust! 🦀🎉

## Next Steps

After completing this course, consider:

1. **Contribute to open source** Rust projects
2. **Build your own systems** using what you've learned
3. **Deep dive** into specific areas (embedded, game dev, crypto)
4. **Share your knowledge** through blog posts or talks
5. **Join the Rust community** on Discord, Reddit, or forums

**Congratulations on completing the Rust Systems Training Course!**
