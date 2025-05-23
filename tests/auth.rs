use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{test, web, App};
use dotenv::dotenv;
use serde_json::json;
use sqlx::PgPool;
use taskforge::models::{TaskPriority, TaskStatus};
use taskforge::routes; // For routes::config
use taskforge::routes::health; // For the health service // Added dotenv // Added imports for enums

#[actix_rt::test]
async fn test_register_and_login_flow() {
    dotenv().ok(); // Load .env file

    // Debug: Print loaded environment variables
    println!(
        "[DEBUG TEST_ENV] DATABASE_URL: {:?}",
        std::env::var("DATABASE_URL")
    );
    println!(
        "[DEBUG TEST_ENV] JWT_SECRET: {:?}",
        std::env::var("JWT_SECRET")
    );

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let _jwt_secret_for_test_setup =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set for tests"); // Not strictly needed for app, but good to check

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // Clean up potential existing user
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind("integration@example.com")
        .execute(&pool)
        .await;

    // Inline App setup
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
            .wrap(Logger::default()) // Ensure Logger is here
            .service(health::health) // health is outside /api and AuthMiddleware
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware) // Apply AuthMiddleware here
                    .configure(routes::config),
            ),
    )
    .await;

    // Register a new user
    let register_payload = json!({
        "username": "integration_user",
        "email": "integration@example.com",
        "password": "Password123!" // Ensure this meets password criteria if any
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body_bytes = test::read_body(resp).await; // Read body for potential error message
    assert_eq!(
        status,
        actix_web::http::StatusCode::CREATED,
        "Registration failed. Body: {:?}",
        String::from_utf8_lossy(&body_bytes)
    );

    // Try to register the same user again (should fail)
    let req_conflict = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload) // Use the same payload
        .to_request();
    let resp_conflict = test::call_service(&app, req_conflict).await;
    let status_conflict = resp_conflict.status();
    let body_bytes_conflict = test::read_body(resp_conflict).await;
    assert_eq!(
        status_conflict,
        actix_web::http::StatusCode::BAD_REQUEST,
        "Duplicate email registration did not fail as expected with 400. Body: {:?}",
        String::from_utf8_lossy(&body_bytes_conflict)
    );
    let error_response_email_conflict: serde_json::Value =
        serde_json::from_slice(&body_bytes_conflict).unwrap();
    assert_eq!(
        error_response_email_conflict["error"],
        "Email already registered"
    );

    // Try to register another user with the same username but different email (should also fail)
    let register_payload_username_conflict = json!({
        "username": "integration_user", // Same username
        "email": "integration_alt@example.com", // Different email
        "password": "Password123!"
    });
    let req_username_conflict = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload_username_conflict)
        .to_request();
    let resp_username_conflict = test::call_service(&app, req_username_conflict).await;
    let status_username_conflict = resp_username_conflict.status();
    let body_bytes_username_conflict = test::read_body(resp_username_conflict).await;
    assert_eq!(
        status_username_conflict,
        actix_web::http::StatusCode::BAD_REQUEST, // Expecting 400 due to refined error handling
        "Duplicate username registration did not fail as expected with 400. Body: {:?}",
        String::from_utf8_lossy(&body_bytes_username_conflict)
    );
    let error_response_username_conflict: serde_json::Value =
        serde_json::from_slice(&body_bytes_username_conflict).unwrap();
    assert_eq!(
        error_response_username_conflict["error"],
        "Username already taken"
    );

    // Login with the registered user
    let login_payload = json!({
        "email": "integration@example.com",
        "password": "Password123!"
    });
    let req_login = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp_login = test::call_service(&app, req_login).await;
    let status_login = resp_login.status();
    // For the login success case, we *expect* a JSON body, so we'll parse it later for token assertion.
    // For the assert_eq message, we can still use from_utf8_lossy or attempt a specific deserialization if the test fails.
    // However, the primary goal is to check the token *after* confirming status is OK.
    // The previous read_body_json for the token is separate from this assert.
    let body_bytes_login = test::read_body(resp_login).await; // This consumes the body.

    assert_eq!(
        status_login,
        actix_web::http::StatusCode::OK,
        "Login failed. Body: {:?}",
        String::from_utf8_lossy(&body_bytes_login)
    );

    // Now, deserialize body_bytes_login for token check
    let login_response: taskforge::auth::AuthResponse =
        serde_json::from_slice(&body_bytes_login).expect("Failed to parse login response JSON");

    let token = login_response.token.clone(); // Clone token for use
    let user_id_from_login = login_response.user_id;

    assert!(!token.is_empty(), "Token should be a non-empty string");

    // 3. Use the token to access a protected route (e.g., create a task)
    let create_task_payload = json!({
        "title": "Task created by token test",
        "status": TaskStatus::Todo, // Using the enum variant
        "priority": TaskPriority::Medium // Adding optional priority for thoroughness
    });

    let req_create_task = test::TestRequest::post()
        .uri("/api/tasks")
        .append_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_task_payload)
        .to_request();

    let resp_create_task = test::call_service(&app, req_create_task).await;
    let status_create_task = resp_create_task.status();
    let body_bytes_create_task = test::read_body(resp_create_task).await;

    assert_eq!(
        status_create_task,
        actix_web::http::StatusCode::CREATED,
        "Create task with token failed. Expected 201, got {}. Body: {:?}",
        status_create_task,
        String::from_utf8_lossy(&body_bytes_create_task)
    );

    // Optionally, deserialize the created task and check its properties
    let created_task_response: serde_json::Value = serde_json::from_slice(&body_bytes_create_task)
        .expect("Failed to parse create task response JSON");
    assert_eq!(
        created_task_response.get("title").and_then(|t| t.as_str()),
        Some("Task created by token test")
    );
    assert_eq!(
        created_task_response.get("status").and_then(|s| s.as_str()),
        Some("todo") // Assuming TaskStatus::Todo serializes to "todo"
    );
    assert_eq!(
        created_task_response
            .get("priority")
            .and_then(|p| p.as_str()),
        Some("medium") // Assuming TaskPriority::Medium serializes to "medium"
    );
    assert_eq!(
        created_task_response
            .get("user_id")
            .and_then(|uid| uid.as_i64()),
        Some(user_id_from_login as i64)
    );

    // Clean up created user
    let _ = sqlx::query("DELETE FROM users WHERE email = $1 OR email = $2")
        .bind("integration@example.com")
        .bind("integration_alt@example.com") // Also clean up the alt email if it somehow got created
        .execute(&pool)
        .await;
}

#[actix_rt::test]
async fn test_invalid_registration_inputs() {
    dotenv().ok(); // Load .env file
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // Inline App setup
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
            .wrap(Logger::default()) // Ensure Logger is here
            .service(health::health)
            .service(web::scope("/api").configure(routes::config)),
    )
    .await;

    let test_cases = vec![
        // Deserialization errors (expect 400 for missing fields)
        (
            json!({ "email": "test@example.com", "password": "Password123!" }),
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing username",
        ),
        (
            json!({ "username": "testuser", "password": "Password123!" }),
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing email",
        ),
        (
            json!({ "username": "testuser", "email": "test@example.com" }),
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing password",
        ),
        // Validation errors (expect 422 for invalid formats/lengths after successful deserialization)
        (
            json!({ "username": "testuser", "email": "invalid-email", "password": "Password123!" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "invalid email format",
        ),
        (
            json!({ "username": "u", "email": "test@example.com", "password": "Password123!" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "username too short",
        ),
        (
            json!({ "username": "a".repeat(33), "email": "test@example.com", "password": "Password123!" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "username too long",
        ),
        (
            json!({ "username": "user name!", "email": "test@example.com", "password": "Password123!" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "username with invalid chars", // This depends on USERNAME_REGEX
        ),
        (
            json!({ "username": "testuser", "email": "test@example.com", "password": "123" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "password too short",
        ),
    ];

    for (payload, expected_status, description) in test_cases {
        let req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;

        assert_eq!(
            status,
            expected_status,
            "Test case failed: {}. Expected {}, got {}. Body: {:?}",
            description,
            expected_status,
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
}

#[actix_rt::test]
async fn test_invalid_login_inputs() {
    dotenv().ok(); // Load .env file
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // --- Setup a valid user for some test cases ---
    let valid_user_email = "login_test_user@example.com";
    let valid_user_password = "Password123!";

    // Clean up potential existing user first
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(valid_user_email)
        .execute(&pool)
        .await;

    // Register the user for tests that require an existing user
    let app_for_setup = test::init_service(
        // Temporary app instance for setup
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default()) // Minimal middleware for setup
            .service(web::scope("/api").configure(routes::config)),
    )
    .await;

    let register_payload = json!({
        "username": "login_test_user",
        "email": valid_user_email,
        "password": valid_user_password
    });
    let reg_req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let reg_resp = test::call_service(&app_for_setup, reg_req).await;
    assert!(
        reg_resp.status().is_success(),
        "Setup: Failed to register test user"
    );
    // --- End user setup ---

    // Main app instance for login tests
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
            .service(web::scope("/api").configure(routes::config)),
    )
    .await;

    let test_cases = vec![
        // Deserialization errors (expect 400 for missing fields)
        (
            json!({ "password": "Password123!" }),
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing email",
        ),
        (
            json!({ "email": valid_user_email }),
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing password",
        ),
        // Validation errors (expect 422 for invalid formats/lengths after successful deserialization)
        (
            json!({ "email": "invalid-email", "password": "Password123!" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "invalid email format",
        ),
        (
            json!({ "email": valid_user_email, "password": "123" }),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            "password too short",
        ),
        // Authentication errors (expect 401)
        (
            json!({ "email": valid_user_email, "password": "WrongPassword123!" }),
            actix_web::http::StatusCode::UNAUTHORIZED,
            "incorrect password",
        ),
        (
            json!({ "email": "nonexistent@example.com", "password": "Password123!" }),
            actix_web::http::StatusCode::UNAUTHORIZED,
            "non-existent user",
        ),
    ];

    for (payload, expected_status, description) in test_cases {
        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;

        assert_eq!(
            status,
            expected_status,
            "Test case failed: {}. Expected {}, got {}. Body: {:?}",
            description,
            expected_status,
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }

    // Clean up the created test user
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(valid_user_email)
        .execute(&pool)
        .await;
}

#[actix_rt::test]
async fn test_protected_route_with_invalid_tokens() {
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
                    .wrap(taskforge::auth::AuthMiddleware) // Middleware applied here
                    .configure(routes::config),
            ),
    )
    .await;

    // Scenario 1: Expired Token
    let expired_claims = taskforge::auth::Claims {
        sub: 999, // Arbitrary user ID
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
    };
    let jwt_secret_for_test =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret".to_string());
    let expired_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &expired_claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret_for_test.as_bytes()),
    )
    .expect("Failed to create expired token for test");

    let req_expired = test::TestRequest::get()
        .uri("/api/tasks") // Any protected route
        .append_header(("Authorization", format!("Bearer {}", expired_token)))
        .to_request();
    let resp_expired_result = test::try_call_service(&app, req_expired).await;
    assert!(
        resp_expired_result.is_err(),
        "Expected an error for expired token"
    );
    if let Err(e) = resp_expired_result {
        let err_response_expired = e.error_response();
        assert_eq!(
            err_response_expired.status(),
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Access with expired token did not result in 401 Unauthorized"
        );
    } else {
        panic!("Expected error for expired token, but got Ok");
    }

    // Scenario 2: Malformed/Incorrectly Signed Token
    let malformed_token = "this.is.not.a.valid.jwt";
    let req_malformed = test::TestRequest::get()
        .uri("/api/tasks")
        .append_header(("Authorization", format!("Bearer {}", malformed_token)))
        .to_request();
    let resp_malformed_result = test::try_call_service(&app, req_malformed).await;
    assert!(
        resp_malformed_result.is_err(),
        "Expected an error for malformed token"
    );
    if let Err(e) = resp_malformed_result {
        let err_response_malformed = e.error_response();
        assert_eq!(
            err_response_malformed.status(),
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Access with malformed token did not result in 401 Unauthorized"
        );
    } else {
        panic!("Expected error for malformed token, but got Ok");
    }

    // Scenario 3: Token signed with a different secret
    let other_secret = "a_completely_different_secret_for_this_token";
    let claims_for_wrong_secret = taskforge::auth::Claims {
        sub: 998,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let token_wrong_secret = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims_for_wrong_secret,
        &jsonwebtoken::EncodingKey::from_secret(other_secret.as_bytes()),
    )
    .expect("Failed to create token with wrong secret");

    let req_wrong_secret = test::TestRequest::get()
        .uri("/api/tasks")
        .append_header(("Authorization", format!("Bearer {}", token_wrong_secret)))
        .to_request();
    let resp_wrong_secret_result = test::try_call_service(&app, req_wrong_secret).await;
    assert!(
        resp_wrong_secret_result.is_err(),
        "Expected an error for token with wrong secret"
    );
    if let Err(e) = resp_wrong_secret_result {
        let err_response_wrong_secret = e.error_response();
        assert_eq!(
            err_response_wrong_secret.status(),
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Access with token signed by different secret did not result in 401 Unauthorized"
        );
    } else {
        panic!("Expected error for token with wrong secret, but got Ok");
    }
}
