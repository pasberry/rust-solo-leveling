# REST API Task Manager

A production-quality REST API for task management built with Axum, SQLite, and async Rust.

## Features

- ✅ Full CRUD operations for tasks
- ✅ SQLite database with migrations
- ✅ Input validation
- ✅ Filtering and pagination
- ✅ Structured logging
- ✅ Comprehensive error handling
- ✅ Unit and integration tests

## Quick Start

### Prerequisites

- Rust 1.70+
- SQLite3

### Setup

1. Clone and navigate to the project:
```bash
cd module-02-async-networking/solutions/ex02-rest-api
```

2. Copy environment file:
```bash
cp .env.example .env
```

3. Build and run:
```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### Health Check
```bash
GET /health
```

### Create Task
```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Write documentation",
    "description": "Complete API docs",
    "status": "Todo",
    "priority": "High"
  }'
```

### List Tasks
```bash
# All tasks
curl http://localhost:3000/api/tasks

# With filters
curl "http://localhost:3000/api/tasks?status=Todo&priority=High&page=1&per_page=10"
```

### Get Single Task
```bash
curl http://localhost:3000/api/tasks/1
```

### Update Task
```bash
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated title",
    "status": "InProgress"
  }'
```

### Toggle Completion
```bash
curl -X PATCH http://localhost:3000/api/tasks/1/complete
```

### Delete Task
```bash
curl -X DELETE http://localhost:3000/api/tasks/1
```

## Testing

Run all tests:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Project Structure

```
.
├── Cargo.toml
├── .env.example
├── migrations/
│   └── 001_create_tasks.sql
└── src/
    ├── main.rs         # Server setup and routing
    ├── config.rs       # Configuration management
    ├── db.rs           # Database connection pool
    ├── models.rs       # Data models and DTOs
    ├── handlers.rs     # HTTP request handlers
    └── error.rs        # Error types and responses
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| DATABASE_URL | SQLite database path | `sqlite:tasks.db` |
| PORT | Server port | `3000` |
| RUST_LOG | Logging level | `rest_api_tasks=debug` |

## Data Model

### Task

| Field | Type | Description |
|-------|------|-------------|
| id | i64 | Unique identifier |
| title | String | Task title (1-200 chars) |
| description | Option<String> | Optional description |
| status | TaskStatus | Todo, InProgress, or Done |
| priority | Priority | Low, Medium, or High |
| completed | bool | Completion status |
| created_at | DateTime<Utc> | Creation timestamp |
| updated_at | DateTime<Utc> | Last update timestamp |

## Error Responses

All errors return JSON:

```json
{
  "error": "Error message",
  "details": {
    "field": "Validation error"
  }
}
```

HTTP Status Codes:
- `400` - Validation error
- `404` - Resource not found
- `500` - Internal server error

## Development

### Database Migrations

Migrations are automatically applied on startup. To create a new migration:

1. Create a new SQL file in `migrations/` with format `NNN_description.sql`
2. Migrations run in order by filename

### Adding New Features

1. Update models in `models.rs`
2. Add handlers in `handlers.rs`
3. Register routes in `main.rs`
4. Add tests

## License

MIT
