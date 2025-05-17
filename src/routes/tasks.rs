use crate::{
    auth::get_user_id,
    error::AppError,
    models::{Task, TaskInput, TaskQuery},
};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

/// Get all tasks
///
/// Returns a list of tasks filtered by the provided query parameters.
#[get("")]
#[allow(unused_assignments)]
pub async fn get_tasks(
    pool: web::Data<PgPool>,
    query_params: web::Query<TaskQuery>,
) -> Result<impl Responder, AppError> {
    let mut sql = String::from(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to \
         FROM tasks WHERE 1=1"
    );
    let mut param_count = 1;

    // Use a temporary vector to hold string representations for ILIKE, or other string-based conditions
    // For direct enum/int binding, we'll bind them directly to the query object.
    
    let mut conditions: Vec<String> = Vec::new();
    
    if query_params.status.is_some() {
        conditions.push(format!("status = ${}", param_count));
        param_count += 1;
    }
    if query_params.priority.is_some() {
        conditions.push(format!("priority = ${}", param_count));
        param_count += 1;
    }
    if query_params.assigned_to.is_some() {
        conditions.push(format!("assigned_to = ${}", param_count));
        param_count += 1;
    }
    if query_params.created_by.is_some() {
        conditions.push(format!("created_by = ${}", param_count));
        param_count += 1;
    }
    if query_params.search.is_some() {
        conditions.push(format!("(title ILIKE ${}", param_count));
        param_count += 1;
        // For the second part of ILIKE, it reuses the same search term but needs a new placeholder
        conditions.last_mut().unwrap().push_str(&format!(" OR description ILIKE ${})", param_count));
        param_count += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }
    
    sql.push_str(" ORDER BY created_at DESC");

    let mut query_builder = sqlx::query_as::<_, Task>(&sql);

    if let Some(status) = &query_params.status {
        query_builder = query_builder.bind(status);
    }
    if let Some(priority) = &query_params.priority {
        query_builder = query_builder.bind(priority);
    }
    if let Some(assigned_to) = query_params.assigned_to {
        query_builder = query_builder.bind(assigned_to);
    }
    if let Some(created_by) = query_params.created_by {
        query_builder = query_builder.bind(created_by);
    }
    if let Some(search) = &query_params.search {
        let search_pattern = format!("%{}%", search);
        query_builder = query_builder.bind(search_pattern.clone()); // For title ILIKE
        query_builder = query_builder.bind(search_pattern);         // For description ILIKE
    }
    
    let tasks = query_builder.fetch_all(&**pool).await?;

    Ok(HttpResponse::Ok().json(tasks))
}

/// Create a new task
///
/// Creates a new task and returns the created task.
#[post("")]
pub async fn create_task(
    pool: web::Data<PgPool>,
    task_data: web::Json<TaskInput>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    // Validate input
    task_data.validate()?;

    let user_id = get_user_id(&req)?;
    let task = Task::new(task_data.into_inner(), user_id);

    // Insert task
    let result = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (id, title, description, priority, status, due_date, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to"
    )
    .bind(task.id)
    .bind(task.title)
    .bind(task.description)
    .bind(task.priority)
    .bind(task.status)
    .bind(task.due_date)
    .bind(task.created_by)
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(result))
}

/// Get a task by ID
///
/// Returns a task by its ID.
#[get("/{id}")]
pub async fn get_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let task = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to 
         FROM tasks WHERE id = $1"
    )
    .bind(task_id.into_inner())
    .fetch_optional(&**pool)
    .await?;

    match task {
        Some(task) => Ok(HttpResponse::Ok().json(task)),
        None => Err(AppError::NotFound("Task not found".into())),
    }
}

/// Update a task
///
/// Updates a task by its ID and returns the updated task.
#[put("/{id}")]
pub async fn update_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
    task_data: web::Json<TaskInput>,
) -> Result<impl Responder, AppError> {
    // Validate input
    task_data.validate()?;

    let result = sqlx::query_as::<_, Task>(
        "UPDATE tasks 
         SET title = $1, description = $2, priority = $3, status = $4, due_date = $5
         WHERE id = $6
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to"
    )
    .bind(&task_data.title)
    .bind(&task_data.description)
    .bind(&task_data.priority)
    .bind(&task_data.status)
    .bind(task_data.due_date)
    .bind(task_id.into_inner())
    .fetch_optional(&**pool)
    .await?;

    match result {
        Some(task) => Ok(HttpResponse::Ok().json(task)),
        None => Err(AppError::NotFound("Task not found".into())),
    }
}

/// Delete a task
///
/// Deletes a task by its ID.
#[delete("/{id}")]
pub async fn delete_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let result = sqlx::query!("DELETE FROM tasks WHERE id = $1", task_id.into_inner())
        .execute(&**pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found".into()));
    }

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskPriority, TaskStatus}; // Make sure enums are in scope
    use actix_web::test;
    use serde_json::json;
    use sqlx::PgPool;
    use std::env;

    // TODO: Fix DB connection for this test or move to integration tests.
    #[ignore]
    #[actix_rt::test]
    async fn test_create_task_validation() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL not set"))
            .await
            .unwrap();

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(pool.clone())) // Ensure pool is cloned if needed elsewhere or use app_data_factory
                .service(create_task),
        )
        .await;

        // Test empty title
        let req = test::TestRequest::post()
            .uri("/tasks") // Assuming create_task is mounted at /tasks (or some prefix)
            .set_json(json!({
                "title": "", // Invalid: empty
                "description": "Test Description",
                "priority": TaskPriority::High, // Using enum
                "status": TaskStatus::Todo      // Using enum
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error()); // Expect 422 for validation error
        
        // Test title too long
        let long_title = "a".repeat(201);
        let req_long_title = test::TestRequest::post()
            .uri("/tasks")
            .set_json(json!({
                "title": long_title,
                "description": "Test Description",
                "priority": TaskPriority::Medium,
                "status": TaskStatus::InProgress
            }))
            .to_request();
        let resp_long_title = test::call_service(&app, req_long_title).await;
        assert!(resp_long_title.status().is_client_error()); // Expect 422 for validation error
    }
}
