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
                .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header().max_age(3600))
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

    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED,
                 "Expected 401 Unauthorized, got {}. Body: {:?}",
                 resp.status(), resp.text().await.unwrap_or_else(|_| "<failed to read body>".to_string()));

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
