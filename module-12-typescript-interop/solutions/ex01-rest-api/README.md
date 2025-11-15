# REST API with TypeScript Client

A production-ready REST API built with Rust (Axum) and TypeScript client.

## Features

- **CRUD Operations**: Complete user management API
- **Type-Safe Client**: TypeScript client with full type safety
- **Error Handling**: Comprehensive error responses
- **Validation**: Input validation (email format)
- **CORS**: Configured for cross-origin requests
- **Pagination**: List endpoints support limit/offset
- **Tests**: Integration tests for all endpoints

## API Endpoints

### Users

- `POST /api/users` - Create user
- `GET /api/users` - List users (supports ?limit=10&offset=0)
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

### Health

- `GET /health` - Health check

## Running the Server

```bash
cargo run
```

Server runs on `http://localhost:8000`

## Testing with curl

```bash
# Health check
curl http://localhost:8000/health

# Create user
curl -X POST http://localhost:8000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# List users
curl http://localhost:8000/api/users

# Get user (replace ID)
curl http://localhost:8000/api/users/SOME-UUID-HERE

# Update user
curl -X PUT http://localhost:8000/api/users/SOME-UUID-HERE \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Smith"}'

# Delete user
curl -X DELETE http://localhost:8000/api/users/SOME-UUID-HERE
```

## TypeScript Client

The `client/` directory contains a type-safe TypeScript client:

- `api-client.ts` - Main client implementation
- `example.ts` - Usage examples

### Usage

```typescript
import { ApiClient } from './api-client';

const client = new ApiClient('http://localhost:8000');

// Create user
const user = await client.users.create({
  name: 'Alice',
  email: 'alice@example.com',
});

// List users
const users = await client.users.list({ limit: 10 });

// Get user
const user = await client.users.get(userId);

// Update user
const updated = await client.users.update(userId, {
  name: 'New Name',
});

// Delete user
await client.users.delete(userId);
```

## Running Tests

```bash
cargo test
```

All 4 integration tests should pass:
- Health check endpoint
- User creation
- Email validation
- User listing
