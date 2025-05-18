use crate::{
    auth::get_user_id,
    error::AppError,
    models::{Task, TaskInput, TaskQuery},
};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

/// Retrieves a list of tasks for the authenticated user.
///
/// This endpoint fetches tasks owned by the authenticated user. It supports
/// filtering by `status`, `priority`, `assigned_to` (user ID), and a `search` term
/// which looks for matches in task titles and descriptions.
/// Tasks are ordered by creation date in descending order.
///
/// ## Query Parameters:
/// - `status` (optional): Filters tasks by their status (e.g., "todo", "inprogress", "done").
/// - `priority` (optional): Filters tasks by their priority (e.g., "low", "medium", "high").
/// - `assigned_to` (optional): Filters tasks by the ID of the user they are assigned to.
/// - `search` (optional): A string to search for in task titles and descriptions (case-insensitive).
///
/// ## Responses:
/// - `200 OK`: Returns a JSON array of `Task` objects.
/// - `401 Unauthorized`: If the request lacks a valid authentication token.
/// - `500 Internal Server Error`: For database errors or other unexpected issues.
#[get("")]
#[allow(unused_assignments)]
pub async fn get_tasks(
    pool: web::Data<PgPool>,
    query_params: web::Query<TaskQuery>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let authenticated_user_id = get_user_id(&req)?;

    // Base query to select tasks for the authenticated user.
    // Conditions for status, priority, assigned_to, and search terms are dynamically appended.
    let mut sql = String::from(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, user_id, assigned_to \
         FROM tasks WHERE user_id = $1"
    );
    let mut param_count = 2;

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
    if query_params.search.is_some() {
        conditions.push(format!("(title ILIKE ${}", param_count));
        param_count += 1;
        conditions
            .last_mut()
            .unwrap()
            .push_str(&format!(" OR description ILIKE ${})", param_count));
        param_count += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC");

    let mut query_builder = sqlx::query_as::<_, Task>(&sql);

    query_builder = query_builder.bind(authenticated_user_id);

    if let Some(status) = &query_params.status {
        query_builder = query_builder.bind(status);
    }
    if let Some(priority) = &query_params.priority {
        query_builder = query_builder.bind(priority);
    }
    if let Some(assigned_to) = query_params.assigned_to {
        query_builder = query_builder.bind(assigned_to);
    }
    if let Some(search) = &query_params.search {
        let search_pattern = format!("%{}%", search);
        query_builder = query_builder.bind(search_pattern.clone());
        query_builder = query_builder.bind(search_pattern);
    }

    let tasks = query_builder.fetch_all(&**pool).await?;

    Ok(HttpResponse::Ok().json(tasks))
}

/// Creates a new task for the authenticated user.
///
/// This endpoint allows an authenticated user to create a new task.
/// It expects a JSON payload conforming to `TaskInput`.
/// The `user_id` of the task is automatically set to the ID of the authenticated user.
///
/// ## Request Body:
/// A JSON object matching the `TaskInput` struct, including:
/// - `title`: The title of the task (required).
/// - `description` (optional): A description of the task.
/// - `priority` (optional): The priority of the task (e.g., "low", "medium", "high").
/// - `status`: The status of the task (e.g., "todo", "inprogress", "done"). Defaults to "todo".
/// - `due_date` (optional): The due date for the task.
///
/// ## Responses:
/// - `201 Created`: Returns the newly created `Task` object as JSON.
/// - `400 Bad Request`: If the input data is invalid (e.g., missing required fields in a way not caught by `validate`).
/// - `401 Unauthorized`: If the request lacks a valid authentication token.
/// - `422 Unprocessable Entity`: If input validation on `TaskInput` fails (e.g., title too short).
/// - `500 Internal Server Error`: For database errors or other unexpected issues.
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
        "INSERT INTO tasks (id, title, description, priority, status, due_date, user_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, user_id, assigned_to"
    )
    .bind(task.id)
    .bind(task.title)
    .bind(task.description)
    .bind(task.priority)
    .bind(task.status)
    .bind(task.due_date)
    .bind(task.user_id)
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(result))
}

/// Retrieves a specific task by its ID.
///
/// This endpoint fetches a single task by its UUID.
/// The authenticated user must be the owner of the task.
///
/// ## Path Parameters:
/// - `id`: The UUID of the task to retrieve.
///
/// ## Responses:
/// - `200 OK`: Returns the `Task` object as JSON if found and owned by the user.
/// - `401 Unauthorized`: If the request lacks a valid authentication token.
/// - `404 Not Found`: If the task with the given ID does not exist or is not owned by the authenticated user.
/// - `500 Internal Server Error`: For database errors or other unexpected issues.
#[get("/{id}")]
pub async fn get_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let authenticated_user_id = get_user_id(&req)?;
    let task_uuid = task_id.into_inner();

    let task = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, user_id, assigned_to 
         FROM tasks WHERE id = $1"
    )
    .bind(task_uuid)
    .fetch_optional(&**pool)
    .await?;

    match task {
        Some(task) => {
            if task.user_id != authenticated_user_id {
                Err(AppError::NotFound("Task not found".into()))
            } else {
                Ok(HttpResponse::Ok().json(task))
            }
        }
        None => Err(AppError::NotFound("Task not found".into())),
    }
}

/// Updates an existing task.
///
/// This endpoint allows an authenticated user to update a task they own.
/// It expects a JSON payload conforming to `TaskInput` and the task's UUID in the path.
/// Only the owner of the task can update it.
///
/// ## Path Parameters:
/// - `id`: The UUID of the task to update.
///
/// ## Request Body:
/// A JSON object matching the `TaskInput` struct, containing the fields to be updated.
/// See `create_task` for details on `TaskInput` fields.
///
/// ## Responses:
/// - `200 OK`: Returns the updated `Task` object as JSON.
/// - `401 Unauthorized`: If the request lacks a valid authentication token.
/// - `404 Not Found`: If the task with the given ID does not exist or is not owned by the authenticated user.
/// - `422 Unprocessable Entity`: If input validation on `TaskInput` fails.
/// - `500 Internal Server Error`: For database errors or other unexpected issues.
#[put("/{id}")]
pub async fn update_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
    task_data: web::Json<TaskInput>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    task_data.validate()?;
    let authenticated_user_id = get_user_id(&req)?;
    let task_uuid = task_id.into_inner();

    // First, verify ownership
    let ownership_check = sqlx::query_as::<_, (i32,)>("SELECT user_id FROM tasks WHERE id = $1")
        .bind(task_uuid)
        .fetch_optional(&**pool)
        .await?;

    match ownership_check {
        Some((owner_user_id,)) => {
            if owner_user_id != authenticated_user_id {
                return Err(AppError::NotFound(
                    "Task not found or not owned by user".into(),
                ));
            }
        }
        None => return Err(AppError::NotFound("Task not found".into())),
    }

    // If ownership is verified, proceed with update
    let result = sqlx::query_as::<_, Task>(
        "UPDATE tasks 
         SET title = $1, description = $2, priority = $3, status = $4, due_date = $5
         WHERE id = $6 AND user_id = $7
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, user_id, assigned_to"
    )
    .bind(&task_data.title)
    .bind(&task_data.description)
    .bind(&task_data.priority)
    .bind(&task_data.status)
    .bind(task_data.due_date)
    .bind(task_uuid)
    .bind(authenticated_user_id)
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(result))
}

/// Deletes a task by its ID.
///
/// This endpoint allows an authenticated user to delete a task they own.
/// Only the owner of the task can delete it.
///
/// ## Path Parameters:
/// - `id`: The UUID of the task to delete.
///
/// ## Responses:
/// - `204 No Content`: On successful deletion.
/// - `401 Unauthorized`: If the request lacks a valid authentication token.
/// - `404 Not Found`: If the task with the given ID does not exist or is not owned by the authenticated user.
/// - `500 Internal Server Error`: For database errors or other unexpected issues.
#[delete("/{id}")]
pub async fn delete_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let authenticated_user_id = get_user_id(&req)?;
    let task_uuid = task_id.into_inner();

    let result = sqlx::query!(
        "DELETE FROM tasks WHERE id = $1 AND user_id = $2",
        task_uuid,
        authenticated_user_id
    )
    .execute(&**pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Task not found or not owned by user".into(),
        ));
    }

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use crate::models::{TaskInput, TaskPriority, TaskStatus};
    use validator::Validate; // For .validate() method

    // No longer async, no actix_rt needed.
    // Remove #[ignore] as it should now pass as a unit test.
    #[test]
    fn test_task_input_validation() {
        // Renamed for clarity
        // Test empty title
        let invalid_input_empty_title = TaskInput {
            title: "".to_string(),
            description: Some("Test Description".to_string()),
            priority: Some(TaskPriority::High),
            status: TaskStatus::Todo,
            due_date: None,
        };
        assert!(
            invalid_input_empty_title.validate().is_err(),
            "Validation should fail for empty title."
        );

        // Test title too long (max 200 according to TaskInput struct)
        let long_title = "a".repeat(201);
        let invalid_input_long_title = TaskInput {
            title: long_title,
            description: Some("Test Description".to_string()),
            priority: Some(TaskPriority::Medium),
            status: TaskStatus::InProgress,
            due_date: None,
        };
        assert!(
            invalid_input_long_title.validate().is_err(),
            "Validation should fail for overly long title."
        );

        // Test valid input
        let valid_input = TaskInput {
            title: "Valid Title".to_string(),
            description: Some("Test Description".to_string()),
            priority: Some(TaskPriority::Low),
            status: TaskStatus::Done,
            due_date: None,
        };
        assert!(
            valid_input.validate().is_ok(),
            "Validation should pass for valid input."
        );

        // Test description too long (max 1000 according to TaskInput struct)
        let long_description = "b".repeat(1001);
        let invalid_input_long_desc = TaskInput {
            title: "Valid title for desc test".to_string(),
            description: Some(long_description),
            priority: Some(TaskPriority::Low),
            status: TaskStatus::Todo,
            due_date: None,
        };
        assert!(
            invalid_input_long_desc.validate().is_err(),
            "Validation should fail for overly long description."
        );

        // Test for priority and status fields (they are enums, no length validation now, but presence might be tested if they weren't Option<Enum>)
        // Since TaskInput.priority is Option<TaskPriority> and TaskInput.status is TaskStatus (not optional),
        // their validation is mostly about type correctness and presence for status, which serde handles at deserialization.
        // The `Validate` derive on TaskInput primarily handles the string length constraints on title and description.
    }
}
