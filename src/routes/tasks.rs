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
    query: web::Query<TaskQuery>,
) -> Result<impl Responder, AppError> {
    // Build the query dynamically based on filters
    let mut sql = String::from(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to \
         FROM tasks WHERE 1=1"
    );
    let mut params: Vec<String> = Vec::new();

    let mut param_count = 1;

    if let Some(status) = &query.status {
        sql.push_str(&format!(" AND status = ${}", param_count));
        params.push(status.clone());
        param_count += 1;
    }

    if let Some(priority) = &query.priority {
        sql.push_str(&format!(" AND priority = ${}", param_count));
        params.push(priority.clone());
        param_count += 1;
    }

    if let Some(assigned_to) = query.assigned_to {
        sql.push_str(&format!(" AND assigned_to = ${}", param_count));
        params.push(assigned_to.to_string());
        param_count += 1;
    }

    if let Some(created_by) = query.created_by {
        sql.push_str(&format!(" AND created_by = ${}", param_count));
        params.push(created_by.to_string());
        param_count += 1;
    }

    if let Some(search) = &query.search {
        let search_pattern = format!("%{}%", search);
        sql.push_str(&format!(" AND (title ILIKE ${}", param_count));
        params.push(search_pattern.clone());
        param_count += 1;
        sql.push_str(&format!(" OR description ILIKE ${})", param_count));
        params.push(search_pattern);
        param_count += 1;
    }

    sql.push_str(" ORDER BY created_at DESC");

    // Execute the query with parameters
    let mut query = sqlx::query_as::<_, Task>(&sql);
    for param in params {
        query = query.bind(param);
    }

    let tasks = query.fetch_all(&**pool).await?;

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
                .app_data(web::Data::new(pool))
                .service(create_task),
        )
        .await;

        // Test empty title
        let req = test::TestRequest::post()
            .uri("/tasks")
            .set_json(json!({
                "title": "",
                "description": "Test Description",
                "priority": "High",
                "status": "Todo"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
