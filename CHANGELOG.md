# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - YYYY-MM-DD

### Added

- **Task Ownership**: Implemented task ownership, ensuring users can only access and modify their own tasks. This involved:
    - New database migration `add_user_id_to_tasks` to add `user_id` foreign key to `tasks` table.
    - Updated `Task` model and route handlers (`create_task`, `get_tasks`, `get_task`, `update_task`, `delete_task`) to enforce ownership.
    - Subsequent migration `rename_created_by_to_user_id_column` to correct initial `user_id` implementation and data.
- **Task Enums**: Refactored `priority` and `status` fields in `Task`, `TaskInput`, and `TaskQuery` from `String` to Rust enums (`TaskPriority`, `TaskStatus`) mirroring SQL enums. This included:
    - Definition of `TaskPriority` and `TaskStatus` enums in `src/models/task.rs` with `sqlx::Type` and `serde` derivations.
    - Updates to task route handlers and tests to use these enums.
- **Comprehensive Integration Tests**:
    - Added `tests/tasks.rs` with extensive CRUD and authorization tests for tasks.
    - Added `test_invalid_registration_inputs` and `test_invalid_login_inputs` to `tests/auth.rs`.
    - Fixed `test_create_task_unauthorized` by using a real HTTP server and `reqwest` client.
- **Code Documentation**: Added extensive Rustdoc comments across the codebase, including:
    - Models: `src/models/task.rs`, `src/models/user.rs`
    - Auth module: `src/auth/mod.rs`, `src/auth/token.rs`, `src/auth/password.rs`, `src/auth/middleware.rs`
    - Route handlers: `src/routes/auth.rs`, `src/routes/health.rs`, `src/routes/tasks.rs`, `src/routes/mod.rs`
    - Error handling: `src/error.rs`
- **README Updates**: Significantly updated `README.md` with current project structure, setup instructions (including `sqlx-cli`), features, and API endpoint details.
- **Changelog**: Initialized this `CHANGELOG.md` file.
- **Dev Dependencies**: Added `reqwest` for integration testing.
- **Gitignore**: Configured `.gitignore` to track `.sqlx/` and `.vscode/settings.json`, `.vscode/extensions.json` while ignoring the rest of `.vscode/`.

### Changed

- **Project Structure Refactoring**:
    - Consolidated task models (`Task`, `TaskInput`, `TaskQuery`) into `src/models/task.rs`.
    - Consolidated user models (`User`, `UserInput`) into `src/models/user.rs`.
    - Created `src/models/mod.rs` to manage model module visibility.
    - Deleted redundant `src/tasks.rs` and `src/models.rs`.
    - Deleted empty utility directories (`src/config/`, `src/errors/` (file renamed), `src/models/` (old dir), `src/services/`).
    - Renamed `src/errors.rs` to `src/error.rs` and updated imports.
- **Auth Module Refactoring**:
    - Broke down the large `src/auth.rs` into a module structure: `src/auth/mod.rs`, `src/auth/middleware.rs`, `src/auth/token.rs`, `src/auth/password.rs`, `src/auth/routes.rs`.
    - Moved auth route handlers from old `src/routes/auth.rs` to new `src/auth/routes.rs` (then these were moved again to the top-level `src/routes/auth.rs` after discussion, and `src/auth/routes.rs` was deleted).
    - `get_user_id` function in `src/auth/mod.rs` now returns `Result<i32, AppError>`.
    - `AuthResponse` in `src/auth/mod.rs` now derives `Deserialize`.
- **SQL Sanitization Removal**: Deleted `src/security.rs` and its custom SQL sanitization functions (`sanitize_input`, `validate_sql_input`) after confirming use of `sqlx` parameterized queries.
- **`app_factory` Removal**: Resolved compilation issues by inlining `App::new()` logic into `HttpServer::new` closure in `src/main.rs`, removing the `app_factory` function and simplifying `src/lib.rs`.
- **Test Refactoring**:
    - Refactored database-dependent unit tests in `src/routes/auth.rs` and `src/routes/tasks.rs` into pure DTO validation unit tests, removing DB and Actix dependencies.
    - Fixed `auth::token::tests::test_token_expiration` by using a `Mutex` to manage `JWT_SECRET` environment variable for tests.
    - Ensured integration tests in `tests/auth.rs` load environment variables using `dotenv().ok()`.
- **Linter Error Fixes**: Addressed various `rust-analyzer` and `Clippy` warnings throughout the codebase.

### Removed

- Deleted `src/tasks.rs` (contents moved to `src/models/task.rs`).
- Deleted old `src/models.rs` (contents split into `src/models/user.rs` and `src/models/task.rs`).
- Deleted `src/security.rs` (SQL sanitization deemed unnecessary).
- Deleted `src/auth.rs` (refactored into `src/auth/` module).
- Deleted empty directories: `src/config/`, `src/errors/` (old dir, file renamed), `src/models/` (old dir), `src/services/`.

### Fixed

- Corrected `get_tasks` dynamic query construction in `src/routes/tasks.rs` to properly bind enum values and search terms.
- Resolved `rust-analyzer(macro-error)` for `sqlx::query!` by installing `sqlx-cli`, running `cargo sqlx prepare`, and configuring `.vscode/settings.json`.
- Fixed various test failures related to auth, task creation with enums, and task ownership implementation.
- Addressed pre-commit hook issues including Clippy warnings and compile errors. 