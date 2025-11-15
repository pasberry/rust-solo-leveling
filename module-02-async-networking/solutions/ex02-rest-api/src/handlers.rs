use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::SqlitePool;
use validator::Validate;

use crate::error::{AppError, Result};
use crate::models::*;

// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// Create a new task
pub async fn create_task(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>)> {
    // Validate input
    payload.validate()?;

    let status = payload.status.unwrap_or(TaskStatus::Todo);
    let priority = payload.priority.unwrap_or(Priority::Medium);
    let now = Utc::now();

    let result = sqlx::query!(
        r#"
        INSERT INTO tasks (title, description, status, priority, completed, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)
        "#,
        payload.title,
        payload.description,
        status.to_string(),
        priority.to_string(),
        now.to_rfc3339(),
        now.to_rfc3339()
    )
    .execute(&pool)
    .await?;

    let task_id = result.last_insert_rowid();

    // Fetch the created task
    let task = get_task_by_id(&pool, task_id).await?;

    Ok((StatusCode::CREATED, Json(task)))
}

// Get a single task by ID
pub async fn get_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Task>> {
    let task = get_task_by_id(&pool, id).await?;
    Ok(Json(task))
}

// List tasks with filtering and pagination
pub async fn list_tasks(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<TaskListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = ((page - 1) * per_page) as i64;

    // Build dynamic query
    let mut sql = String::from("SELECT * FROM tasks WHERE 1=1");
    let mut count_sql = String::from("SELECT COUNT(*) as count FROM tasks WHERE 1=1");

    let mut params: Vec<String> = Vec::new();

    if let Some(status) = &query.status {
        sql.push_str(" AND status = ?");
        count_sql.push_str(" AND status = ?");
        params.push(status.to_string());
    }

    if let Some(priority) = &query.priority {
        sql.push_str(" AND priority = ?");
        count_sql.push_str(" AND priority = ?");
        params.push(priority.to_string());
    }

    if let Some(completed) = query.completed {
        sql.push_str(" AND completed = ?");
        count_sql.push_str(" AND completed = ?");
        params.push(if completed { "1" } else { "0" }.to_string());
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    // Get total count
    let total: i64 = sqlx::query_scalar(&count_sql)
        .bind(params.get(0))
        .bind(params.get(1))
        .bind(params.get(2))
        .fetch_one(&pool)
        .await?;

    // Get tasks
    let tasks = sqlx::query_as::<_, Task>(&sql)
        .bind(params.get(0))
        .bind(params.get(1))
        .bind(params.get(2))
        .bind(per_page as i64)
        .bind(offset)
        .fetch_all(&pool)
        .await?;

    Ok(Json(TaskListResponse {
        tasks,
        total,
        page,
        per_page,
    }))
}

// Update a task
pub async fn update_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>> {
    // Validate input
    payload.validate()?;

    // Check if task exists
    get_task_by_id(&pool, id).await?;

    let now = Utc::now();

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(title) = &payload.title {
        updates.push("title = ?");
        params.push(title.clone());
    }

    if let Some(description) = &payload.description {
        updates.push("description = ?");
        params.push(description.clone());
    }

    if let Some(status) = &payload.status {
        updates.push("status = ?");
        params.push(status.to_string());
    }

    if let Some(priority) = &payload.priority {
        updates.push("priority = ?");
        params.push(priority.to_string());
    }

    if updates.is_empty() {
        // Nothing to update, return current task
        return get_task(State(pool), Path(id)).await;
    }

    updates.push("updated_at = ?");
    params.push(now.to_rfc3339());

    let sql = format!(
        "UPDATE tasks SET {} WHERE id = ?",
        updates.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for param in params {
        query = query.bind(param);
    }
    query = query.bind(id);

    query.execute(&pool).await?;

    // Fetch updated task
    let task = get_task_by_id(&pool, id).await?;
    Ok(Json(task))
}

// Delete a task
pub async fn delete_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let result = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// Toggle task completion
pub async fn toggle_complete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Task>> {
    let task = get_task_by_id(&pool, id).await?;
    let new_completed = !task.completed;
    let now = Utc::now();

    sqlx::query!(
        "UPDATE tasks SET completed = ?, updated_at = ? WHERE id = ?",
        new_completed,
        now.to_rfc3339(),
        id
    )
    .execute(&pool)
    .await?;

    let task = get_task_by_id(&pool, id).await?;
    Ok(Json(task))
}

// Helper function to get task by ID
async fn get_task_by_id(pool: &SqlitePool, id: i64) -> Result<Task> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_pool;

    async fn setup_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_task() {
        let pool = setup_db().await;

        let create_req = CreateTaskRequest {
            title: "Test task".to_string(),
            description: Some("Description".to_string()),
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::High),
        };

        let (status, Json(task)) = create_task(State(pool.clone()), Json(create_req))
            .await
            .unwrap();

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::High);

        // Get the task
        let Json(fetched_task) = get_task(State(pool), Path(task.id))
            .await
            .unwrap();

        assert_eq!(fetched_task.id, task.id);
        assert_eq!(fetched_task.title, "Test task");
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let pool = setup_db().await;

        // Create some tasks
        for i in 0..5 {
            let create_req = CreateTaskRequest {
                title: format!("Task {}", i),
                description: None,
                status: Some(if i % 2 == 0 { TaskStatus::Todo } else { TaskStatus::Done }),
                priority: Some(Priority::Medium),
            };

            create_task(State(pool.clone()), Json(create_req))
                .await
                .unwrap();
        }

        // List all tasks
        let query = ListTasksQuery::default();
        let Json(response) = list_tasks(State(pool.clone()), Query(query))
            .await
            .unwrap();

        assert_eq!(response.tasks.len(), 5);
        assert_eq!(response.total, 5);

        // List with filter
        let query = ListTasksQuery {
            status: Some(TaskStatus::Todo),
            ..Default::default()
        };
        let Json(response) = list_tasks(State(pool), Query(query))
            .await
            .unwrap();

        assert_eq!(response.tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_update_task() {
        let pool = setup_db().await;

        let create_req = CreateTaskRequest {
            title: "Original title".to_string(),
            description: None,
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::Low),
        };

        let (_, Json(task)) = create_task(State(pool.clone()), Json(create_req))
            .await
            .unwrap();

        let update_req = UpdateTaskRequest {
            title: Some("Updated title".to_string()),
            description: Some("New description".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: None,
        };

        let Json(updated_task) = update_task(State(pool), Path(task.id), Json(update_req))
            .await
            .unwrap();

        assert_eq!(updated_task.title, "Updated title");
        assert_eq!(updated_task.description, Some("New description".to_string()));
        assert_eq!(updated_task.status, TaskStatus::InProgress);
        assert_eq!(updated_task.priority, Priority::Low); // Unchanged
    }

    #[tokio::test]
    async fn test_delete_task() {
        let pool = setup_db().await;

        let create_req = CreateTaskRequest {
            title: "To be deleted".to_string(),
            description: None,
            status: None,
            priority: None,
        };

        let (_, Json(task)) = create_task(State(pool.clone()), Json(create_req))
            .await
            .unwrap();

        let status = delete_task(State(pool.clone()), Path(task.id))
            .await
            .unwrap();

        assert_eq!(status, StatusCode::NO_CONTENT);

        // Verify task is deleted
        let result = get_task(State(pool), Path(task.id)).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn test_toggle_complete() {
        let pool = setup_db().await;

        let create_req = CreateTaskRequest {
            title: "Task".to_string(),
            description: None,
            status: None,
            priority: None,
        };

        let (_, Json(task)) = create_task(State(pool.clone()), Json(create_req))
            .await
            .unwrap();

        assert!(!task.completed);

        let Json(toggled) = toggle_complete(State(pool.clone()), Path(task.id))
            .await
            .unwrap();

        assert!(toggled.completed);

        let Json(toggled_again) = toggle_complete(State(pool), Path(task.id))
            .await
            .unwrap();

        assert!(!toggled_again.completed);
    }
}
