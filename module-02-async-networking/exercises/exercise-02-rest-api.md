# Exercise 02: REST API with Database

## Objective

Build a production-quality REST API for a task management system with database persistence, authentication, and proper error handling.

**Estimated time:** 4-5 hours

## Requirements

### Core Features

1. **Task Management**
   - Create tasks with title, description, status, priority
   - Update task fields
   - Delete tasks
   - List tasks with filtering and pagination
   - Mark tasks as complete/incomplete

2. **Database Persistence**
   - Use SQLite with sqlx
   - Automatic migrations
   - Connection pooling
   - Proper error handling

3. **REST API Endpoints**
   ```
   POST   /api/tasks              - Create task
   GET    /api/tasks              - List tasks (with filters)
   GET    /api/tasks/:id          - Get single task
   PUT    /api/tasks/:id          - Update task
   DELETE /api/tasks/:id          - Delete task
   PATCH  /api/tasks/:id/complete - Toggle completion
   GET    /health                 - Health check
   ```

4. **Request/Response**
   - JSON request bodies
   - JSON responses
   - Proper HTTP status codes
   - Validation errors returned as JSON

### Technical Requirements

1. **Framework**: Use Axum
2. **Database**: SQLite with sqlx (compile-time checked queries)
3. **Error Handling**: Custom error type with proper HTTP responses
4. **Validation**: Input validation with helpful error messages
5. **Configuration**: Environment variables for database path, port
6. **Logging**: Structured logging with tracing

## Data Model

```rust
struct Task {
    id: i64,
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: Priority,
    completed: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

enum Priority {
    Low,
    Medium,
    High,
}
```

## API Examples

### Create Task
```bash
POST /api/tasks
Content-Type: application/json

{
  "title": "Write documentation",
  "description": "Complete API docs",
  "status": "Todo",
  "priority": "High"
}

Response: 201 Created
{
  "id": 1,
  "title": "Write documentation",
  "description": "Complete API docs",
  "status": "Todo",
  "priority": "High",
  "completed": false,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### List Tasks with Filters
```bash
GET /api/tasks?status=Todo&priority=High&page=1&per_page=10

Response: 200 OK
{
  "tasks": [...],
  "total": 42,
  "page": 1,
  "per_page": 10
}
```

### Error Response
```bash
POST /api/tasks
Content-Type: application/json

{
  "title": ""
}

Response: 400 Bad Request
{
  "error": "Validation failed",
  "details": {
    "title": "Title cannot be empty"
  }
}
```

## Project Structure

```
exercise-02-rest-api/
├── Cargo.toml
├── .env
├── migrations/
│   └── 001_create_tasks.sql
└── src/
    ├── main.rs
    ├── config.rs        # Configuration
    ├── db.rs            # Database setup
    ├── models.rs        # Task model
    ├── handlers.rs      # HTTP handlers
    ├── error.rs         # Error types
    └── validation.rs    # Input validation
```

## Testing Checklist

- [ ] Server starts and connects to database
- [ ] Can create task with valid data
- [ ] Creating task with empty title returns 400
- [ ] Can retrieve task by ID
- [ ] Getting non-existent task returns 404
- [ ] Can update task fields
- [ ] Can delete task
- [ ] Can list tasks with pagination
- [ ] Can filter by status and priority
- [ ] Completing task toggles completed flag
- [ ] Health endpoint returns 200
- [ ] Database connection errors handled gracefully

## Bonus Challenges

1. **Tags**: Add tags to tasks (many-to-many relationship)
2. **Search**: Full-text search in title/description
3. **Due Dates**: Add due_date field with overdue filtering
4. **Sorting**: Support sorting by created_at, priority, etc.
5. **Bulk Operations**: Delete/update multiple tasks
6. **OpenAPI**: Generate OpenAPI/Swagger documentation
7. **Rate Limiting**: Add rate limiting middleware
8. **Caching**: Cache frequently accessed tasks

## Hints

1. **Database Setup**: Use sqlx migrations for schema
2. **Query Builder**: Use `sqlx::query_as!` for compile-time checking
3. **Error Conversion**: Implement `From<sqlx::Error>` for your error type
4. **Validation**: Create a `Validate` trait for request types
5. **State**: Pass `PgPool` in `AppState` using `Arc`

## Example Test

```bash
# Create a task
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test task",
    "description": "Testing the API",
    "status": "Todo",
    "priority": "Medium"
  }'

# List all tasks
curl http://localhost:3000/api/tasks

# Get specific task
curl http://localhost:3000/api/tasks/1

# Update task
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated task",
    "status": "InProgress"
  }'

# Complete task
curl -X PATCH http://localhost:3000/api/tasks/1/complete

# Delete task
curl -X DELETE http://localhost:3000/api/tasks/1
```

## Submission Requirements

- Compiles without warnings
- All tests pass
- Database migrations work correctly
- Proper error handling (no unwrap() in handlers)
- README with setup instructions
- Environment variables documented

Good luck!
