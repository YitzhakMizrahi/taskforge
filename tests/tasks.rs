use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, rt, test, web, App, HttpServer};
use dotenv::dotenv;
use serde_json::json;
use sqlx::PgPool;
use std::net::TcpListener;
use taskforge::models::{Task, TaskPriority, TaskStatus};
use taskforge::routes;
use taskforge::routes::health;
// reqwest client will be used in the test_create_task_unauthorized

// Helper struct to hold auth details
struct TestUser {
    id: i32,
    token: String,
}

async fn register_and_login_user(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    email: &str,
    username: &str,
    password: &str,
) -> Result<TestUser, String> {
    // Register
    let req_register = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .to_request();
    let resp_register = test::call_service(app, req_register).await;
    let resp_status = resp_register.status();
    let auth_response_bytes = test::read_body(resp_register).await;

    if !resp_status.is_success() {
        return Err(format!(
            "Failed to register user. Status: {}. Body: {}",
            resp_status,
            String::from_utf8_lossy(&auth_response_bytes)
        ));
    }
    let auth_response: taskforge::auth::AuthResponse =
        serde_json::from_slice(&auth_response_bytes) // Explicit path for AuthResponse
            .map_err(|e| format!("Failed to parse registration response: {}", e))?;

    Ok(TestUser {
        id: auth_response.user_id,
        token: auth_response.token,
    })
}

async fn cleanup_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

#[actix_rt::test]
async fn test_create_task_unauthorized() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Drop the listener so the server can bind to it

    let server_pool = pool.clone();
    let server_handle = rt::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(server_pool.clone()))
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header()
                        .max_age(3600),
                )
                .wrap(Logger::default())
                .service(health::health)
                .service(
                    web::scope("/api")
                        .wrap(taskforge::auth::AuthMiddleware)
                        .configure(routes::config),
                )
        })
        .bind(("127.0.0.1", port))
        .unwrap_or_else(|_| panic!("Failed to bind to port {}", port))
        .run()
        .await
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let task_payload = json!({
        "title": "Unauthorized Task",
        "status": TaskStatus::Todo
    });

    let request_url = format!("http://127.0.0.1:{}/api/tasks", port);

    let resp = client
        .post(&request_url)
        .json(&task_payload)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized, got {}. Body: {:?}",
        resp.status(),
        resp.text()
            .await
            .unwrap_or_else(|_| "<failed to read body>".to_string())
    );

    // Stop the server by aborting the spawned task
    // Note: server_handle.abort() does not immediately guarantee the server stops listening.
    // For more graceful shutdown, you'd typically use Server::stop() via a handle,
    // but that's more complex for this test scenario.
    // Aborting is generally fine for tests if a bit abrupt.
    server_handle.abort();
    // Optionally, wait for the server to fully shut down, though not strictly necessary for this test
    // let _ = server_handle.await;
}

#[actix_rt::test]
async fn test_task_crud_flow() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app_for_crud = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_email = "crud_user@example.com";
    let user_username = "crud_user";
    let user_password = "PasswordCrud123!";

    cleanup_user(&pool, user_email).await;

    let test_user =
        register_and_login_user(&app_for_crud, user_email, user_username, user_password)
            .await
            .expect("Failed to register/login test user for CRUD flow");

    // 1. Create Task
    let task_payload_create = json!({
        "title": "CRUD Task 1 Original",
        "status": TaskStatus::Todo,
        "description": "Initial description",
        "priority": TaskPriority::Medium
    });
    let req_create = test::TestRequest::post()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&task_payload_create)
        .to_request();
    let resp_create = test::call_service(&app_for_crud, req_create).await;
    assert_eq!(resp_create.status(), actix_web::http::StatusCode::CREATED);
    let created_task: Task = test::read_body_json(resp_create).await;
    assert_eq!(created_task.title, "CRUD Task 1 Original");
    assert_eq!(created_task.status, TaskStatus::Todo);
    assert_eq!(
        created_task.description.as_deref(),
        Some("Initial description")
    );
    assert_eq!(created_task.priority, Some(TaskPriority::Medium));
    assert_eq!(created_task.user_id, test_user.id);
    let task_id_1 = created_task.id;

    // 2. Get Task by ID
    let req_get = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", task_id_1))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_get = test::call_service(&app_for_crud, req_get).await;
    assert_eq!(resp_get.status(), actix_web::http::StatusCode::OK);
    let fetched_task: Task = test::read_body_json(resp_get).await;
    assert_eq!(fetched_task.id, task_id_1);
    assert_eq!(fetched_task.title, "CRUD Task 1 Original");

    // 3. Update Task
    let task_payload_update = json!({
        "title": "CRUD Task 1 Updated",
        "status": TaskStatus::InProgress,
        "description": "Updated description",
        "priority": TaskPriority::High
    });
    let req_update = test::TestRequest::put()
        .uri(&format!("/api/tasks/{}", task_id_1))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&task_payload_update)
        .to_request();
    let resp_update = test::call_service(&app_for_crud, req_update).await;
    assert_eq!(resp_update.status(), actix_web::http::StatusCode::OK);
    let updated_task: Task = test::read_body_json(resp_update).await;
    assert_eq!(updated_task.id, task_id_1);
    assert_eq!(updated_task.title, "CRUD Task 1 Updated");
    assert_eq!(updated_task.status, TaskStatus::InProgress);
    assert_eq!(
        updated_task.description.as_deref(),
        Some("Updated description")
    );
    assert_eq!(updated_task.priority, Some(TaskPriority::High));

    // 4. Create a second task for Get All check
    let task_payload_create2 = json!({
        "title": "CRUD Task 2",
        "status": TaskStatus::Done,
        "priority": TaskPriority::Low
    });
    let req_create2 = test::TestRequest::post()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&task_payload_create2)
        .to_request();
    let resp_create2 = test::call_service(&app_for_crud, req_create2).await;
    assert_eq!(resp_create2.status(), actix_web::http::StatusCode::CREATED);
    let created_task2: Task = test::read_body_json(resp_create2).await;
    let task_id_2 = created_task2.id;

    // 5. Get All Tasks
    let req_get_all = test::TestRequest::get()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_get_all = test::call_service(&app_for_crud, req_get_all).await;
    assert_eq!(resp_get_all.status(), actix_web::http::StatusCode::OK);
    let tasks: Vec<Task> = test::read_body_json(resp_get_all).await;
    assert!(
        tasks.len() >= 2,
        "Expected at least 2 tasks for the user, found {}",
        tasks.len()
    );
    assert!(tasks
        .iter()
        .any(|t| t.id == task_id_1 && t.title == "CRUD Task 1 Updated"));
    assert!(tasks
        .iter()
        .any(|t| t.id == task_id_2 && t.title == "CRUD Task 2"));

    // 6. Delete Task 1
    let req_delete1 = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", task_id_1))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_delete1 = test::call_service(&app_for_crud, req_delete1).await;
    assert_eq!(
        resp_delete1.status(),
        actix_web::http::StatusCode::NO_CONTENT
    );

    // Verify Task 1 is deleted
    let req_get_deleted1 = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", task_id_1))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_get_deleted1 = test::call_service(&app_for_crud, req_get_deleted1).await;
    assert_eq!(
        resp_get_deleted1.status(),
        actix_web::http::StatusCode::NOT_FOUND
    );

    // 7. Delete Task 2
    let req_delete2 = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", task_id_2))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_delete2 = test::call_service(&app_for_crud, req_delete2).await;
    assert_eq!(
        resp_delete2.status(),
        actix_web::http::StatusCode::NO_CONTENT
    );

    cleanup_user(&pool, user_email).await;
}

#[actix_rt::test]
async fn test_task_ownership_and_authorization() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_a_email = "owner_user_a@example.com";
    let user_a_username = "owner_user_a";
    let user_a_password = "PasswordOwnerA123!";

    let user_b_email = "other_user_b@example.com";
    let user_b_username = "other_user_b";
    let user_b_password = "PasswordOtherB123!";

    // Cleanup potential old users first
    cleanup_user(&pool, user_a_email).await;
    cleanup_user(&pool, user_b_email).await;

    // Register and login User A
    let user_a = register_and_login_user(&app, user_a_email, user_a_username, user_a_password)
        .await
        .expect("Failed to register/login User A");

    // Register and login User B
    let user_b = register_and_login_user(&app, user_b_email, user_b_username, user_b_password)
        .await
        .expect("Failed to register/login User B");

    // User A creates a task
    let task_payload_user_a = json!({
        "title": "User A\'s Task",
        "status": TaskStatus::Todo,
        "priority": TaskPriority::High
    });
    let req_create_task_a = test::TestRequest::post()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_a.token)))
        .set_json(&task_payload_user_a)
        .to_request();
    let resp_create_task_a = test::call_service(&app, req_create_task_a).await;
    assert_eq!(
        resp_create_task_a.status(),
        actix_web::http::StatusCode::CREATED,
        "User A failed to create task"
    );
    let task_a: Task = test::read_body_json(resp_create_task_a).await;
    let task_a_id = task_a.id;

    // 1. User B lists tasks: should not see User A's task
    let req_list_tasks_b = test::TestRequest::get()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_b.token)))
        .to_request();
    let resp_list_tasks_b = test::call_service(&app, req_list_tasks_b).await;
    assert_eq!(resp_list_tasks_b.status(), actix_web::http::StatusCode::OK);
    let tasks_for_b: Vec<Task> = test::read_body_json(resp_list_tasks_b).await;
    assert!(
        !tasks_for_b.iter().any(|t| t.id == task_a_id),
        "User B should not see User A\'s task in their list"
    );

    // 2. User B tries to get User A's task by ID: should get 404
    let req_get_task_a_by_b = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", task_a_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_b.token)))
        .to_request();
    let resp_get_task_a_by_b = test::call_service(&app, req_get_task_a_by_b).await;
    assert_eq!(
        resp_get_task_a_by_b.status(),
        actix_web::http::StatusCode::NOT_FOUND,
        "User B should get 404 when trying to fetch User A\'s task by ID"
    );

    // 3. User B tries to update User A's task: should get 404
    let update_payload_by_b = json!({
        "title": "Attempted Update by B",
        "status": TaskStatus::InProgress
    });
    let req_update_task_a_by_b = test::TestRequest::put()
        .uri(&format!("/api/tasks/{}", task_a_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_b.token)))
        .set_json(&update_payload_by_b)
        .to_request();
    let resp_update_task_a_by_b = test::call_service(&app, req_update_task_a_by_b).await;
    assert_eq!(
        resp_update_task_a_by_b.status(),
        actix_web::http::StatusCode::NOT_FOUND, // Or FORBIDDEN, but 404 is good for not leaking info
        "User B should get 404 when trying to update User A\'s task"
    );

    // 4. User B tries to delete User A's task: should get 404
    let req_delete_task_a_by_b = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", task_a_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_b.token)))
        .to_request();
    let resp_delete_task_a_by_b = test::call_service(&app, req_delete_task_a_by_b).await;
    assert_eq!(
        resp_delete_task_a_by_b.status(),
        actix_web::http::StatusCode::NOT_FOUND,
        "User B should get 404 when trying to delete User A\'s task"
    );

    // Verify User A can still fetch their own task (sanity check)
    let req_get_task_a_by_a = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", task_a_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", user_a.token)))
        .to_request();
    let resp_get_task_a_by_a = test::call_service(&app, req_get_task_a_by_a).await;
    assert_eq!(
        resp_get_task_a_by_a.status(),
        actix_web::http::StatusCode::OK,
        "User A should be able to fetch their own task"
    );

    // Cleanup
    cleanup_user(&pool, user_a_email).await;
    cleanup_user(&pool, user_b_email).await;
}

#[actix_rt::test]
async fn test_get_tasks_with_filtering() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_email = "filter_user@example.com";
    let user_username = "filter_user";
    let user_password = "PasswordFilter123!";

    cleanup_user(&pool, user_email).await; // Ensure clean state
    let test_user = register_and_login_user(&app, user_email, user_username, user_password)
        .await
        .expect("Failed to register/login user for filtering tests");

    // --- Create a diverse set of tasks ---
    let tasks_to_create = vec![
        json!({ "title": "Alpha Todo Low", "status": TaskStatus::Todo, "priority": TaskPriority::Low, "description": "Searchable one" }),
        json!({ "title": "Bravo InProgress Medium", "status": TaskStatus::InProgress, "priority": TaskPriority::Medium, "description": "Another task" }),
        json!({ "title": "Charlie Done High", "status": TaskStatus::Done, "priority": TaskPriority::High, "description": "High importance" }),
        json!({ "title": "Delta Todo Medium", "status": TaskStatus::Todo, "priority": TaskPriority::Medium, "description": "Searchable two" }),
        json!({ "title": "Echo Review Urgent", "status": TaskStatus::Review, "priority": TaskPriority::Urgent, "description": "Urgent review needed" }),
    ];

    let mut created_task_ids: Vec<uuid::Uuid> = Vec::new();
    for task_payload in tasks_to_create {
        let req = test::TestRequest::post()
            .uri("/api/tasks")
            .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
            .set_json(&task_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED, "Failed to create task for filtering setup");
        let task: Task = test::read_body_json(resp).await;
        created_task_ids.push(task.id);
    }
    assert_eq!(created_task_ids.len(), 5, "Incorrect number of tasks created for setup");

    // --- Test filtering ---

    // Filter by status: Todo (should be 2 tasks: Alpha, Delta)
    let req_status_todo = test::TestRequest::get()
        .uri("/api/tasks?status=todo")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_status_todo = test::call_service(&app, req_status_todo).await;
    assert_eq!(resp_status_todo.status(), actix_web::http::StatusCode::OK);
    let tasks_status_todo: Vec<Task> = test::read_body_json(resp_status_todo).await;
    assert_eq!(tasks_status_todo.len(), 2);
    assert!(tasks_status_todo.iter().all(|t| t.status == TaskStatus::Todo));

    // Filter by priority: Medium (should be 2 tasks: Bravo, Delta)
    let req_prio_medium = test::TestRequest::get()
        .uri("/api/tasks?priority=medium")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_prio_medium = test::call_service(&app, req_prio_medium).await;
    assert_eq!(resp_prio_medium.status(), actix_web::http::StatusCode::OK);
    let tasks_prio_medium: Vec<Task> = test::read_body_json(resp_prio_medium).await;
    assert_eq!(tasks_prio_medium.len(), 2);
    assert!(tasks_prio_medium.iter().all(|t| t.priority == Some(TaskPriority::Medium)));

    // Filter by search: "Searchable" (should be 2 tasks: Alpha, Delta)
    let req_search = test::TestRequest::get()
        .uri("/api/tasks?search=Searchable") // Case-insensitive in handler, but test with exact case for clarity
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_search = test::call_service(&app, req_search).await;
    assert_eq!(resp_search.status(), actix_web::http::StatusCode::OK);
    let tasks_search: Vec<Task> = test::read_body_json(resp_search).await;
    assert_eq!(tasks_search.len(), 2);
    assert!(tasks_search.iter().any(|t| t.title.contains("Alpha")));
    assert!(tasks_search.iter().any(|t| t.title.contains("Delta")));

    // Filter by search: "Alpha" (should match title of 1 task)
    let req_search_title = test::TestRequest::get()
        .uri("/api/tasks?search=Alpha")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_search_title = test::call_service(&app, req_search_title).await;
    assert_eq!(resp_search_title.status(), actix_web::http::StatusCode::OK);
    let tasks_search_title: Vec<Task> = test::read_body_json(resp_search_title).await;
    assert_eq!(tasks_search_title.len(), 1);
    assert_eq!(tasks_search_title[0].title, "Alpha Todo Low");


    // Filter by status and priority: status=todo & priority=medium (should be 1 task: Delta)
    let req_status_prio = test::TestRequest::get()
        .uri("/api/tasks?status=todo&priority=medium")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_status_prio = test::call_service(&app, req_status_prio).await;
    assert_eq!(resp_status_prio.status(), actix_web::http::StatusCode::OK);
    let tasks_status_prio: Vec<Task> = test::read_body_json(resp_status_prio).await;
    assert_eq!(tasks_status_prio.len(), 1);
    assert_eq!(tasks_status_prio[0].title, "Delta Todo Medium");

    // Filter yielding no results: status=done & priority=low (no such task)
    let req_no_results = test::TestRequest::get()
        .uri("/api/tasks?status=done&priority=low")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_no_results = test::call_service(&app, req_no_results).await;
    assert_eq!(resp_no_results.status(), actix_web::http::StatusCode::OK);
    let tasks_no_results: Vec<Task> = test::read_body_json(resp_no_results).await;
    assert!(tasks_no_results.is_empty());

    // --- Cleanup ---
    for task_id in created_task_ids {
        let _ = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
            .bind(task_id)
            .bind(test_user.id)
            .execute(&pool)
            .await;
    }
    cleanup_user(&pool, user_email).await;
}

#[actix_rt::test]
async fn test_create_task_minimal_fields() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_email = "minimal_user@example.com";
    cleanup_user(&pool, user_email).await; // Ensure clean state
    let test_user = register_and_login_user(&app, user_email, "minimal_user", "PassMinimal123!")
        .await
        .expect("Failed to register/login user for minimal task test");

    let minimal_payload = json!({
        "title": "Minimal Task Title",
        "status": TaskStatus::Todo // Status is mandatory in TaskInput
        // description, priority, due_date are omitted
    });

    let req = test::TestRequest::post()
        .uri("/api/tasks")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&minimal_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED, "Failed to create task with minimal fields");

    let created_task: Task = test::read_body_json(resp).await;
    assert_eq!(created_task.title, "Minimal Task Title");
    assert_eq!(created_task.status, TaskStatus::Todo);
    assert!(created_task.description.is_none());
    assert!(created_task.priority.is_none()); // Default priority is None if not provided
    assert!(created_task.due_date.is_none());
    assert_eq!(created_task.user_id, test_user.id);

    cleanup_user(&pool, user_email).await;
}

#[actix_rt::test]
async fn test_update_non_existent_task() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_email = "non_existent_update_user@example.com";
    cleanup_user(&pool, user_email).await; 
    let test_user = register_and_login_user(&app, user_email, "non_existent_updater", "PassNonExistent1!")
        .await
        .expect("Failed to register/login user for non-existent task update test");

    let non_existent_task_id = uuid::Uuid::new_v4(); // Random, non-existent UUID
    let update_payload = json!({
        "title": "Update for Non-Existent Task",
        "status": TaskStatus::Done
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/tasks/{}", non_existent_task_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&update_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND, "Updating non-existent task did not return 404");

    cleanup_user(&pool, user_email).await;
}

#[actix_rt::test]
async fn test_delete_non_existent_task() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;

    let user_email = "non_existent_delete_user@example.com";
    cleanup_user(&pool, user_email).await;
    let test_user = register_and_login_user(&app, user_email, "non_existent_deleter", "PassNonExistent2!")
        .await
        .expect("Failed to register/login user for non-existent task delete test");

    let non_existent_task_id = uuid::Uuid::new_v4(); // Random, non-existent UUID

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", non_existent_task_id))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND, "Deleting non-existent task did not return 404");

    cleanup_user(&pool, user_email).await;
}

#[actix_rt::test]
async fn test_task_invalid_uuid_format() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(health::health)
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware)
                    .configure(routes::config),
            ),
    )
    .await;
    
    let user_email = "invalid_uuid_user@example.com";
    cleanup_user(&pool, user_email).await;
    let test_user = register_and_login_user(&app, user_email, "invalid_uuid_user", "PassInvalidUuid1!")
        .await
        .expect("Failed to register/login user for invalid uuid test");

    let invalid_uuid = "not-a-valid-uuid";

    // Test GET with invalid UUID
    let req_get = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", invalid_uuid))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_get = test::call_service(&app, req_get).await;
    // Actix path extractor for Uuid usually results in 404 if parsing fails before handler
    assert_eq!(resp_get.status(), actix_web::http::StatusCode::NOT_FOUND, "GET with invalid UUID did not return 404");

    // Test PUT with invalid UUID
    let update_payload = json!({
        "title": "Attempted Update",
        "status": TaskStatus::Todo
    });
    let req_put = test::TestRequest::put()
        .uri(&format!("/api/tasks/{}", invalid_uuid))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .set_json(&update_payload)
        .to_request();
    let resp_put = test::call_service(&app, req_put).await;
    assert_eq!(resp_put.status(), actix_web::http::StatusCode::NOT_FOUND, "PUT with invalid UUID did not return 404");

    // Test DELETE with invalid UUID
    let req_delete = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", invalid_uuid))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", test_user.token)))
        .to_request();
    let resp_delete = test::call_service(&app, req_delete).await;
    assert_eq!(resp_delete.status(), actix_web::http::StatusCode::NOT_FOUND, "DELETE with invalid UUID did not return 404");

    cleanup_user(&pool, user_email).await;
}
