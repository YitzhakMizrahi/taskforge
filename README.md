# TaskForge API

A RESTful API for task management built with Rust and Actix Web.

## Features

- User authentication with JWT tokens
- Task management (CRUD operations)
- Task ownership (users can only manage their own tasks)
- Enum-based task priority and status
- Input validation
- Comprehensive error handling
- Database integration with PostgreSQL using SQLx
- Asynchronous request processing

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Cargo
- `sqlx-cli` (for database migrations and query preparation)
  ```bash
  cargo install sqlx-cli --no-default-features --features native-tls,postgres
  ```

## Environment Variables

Create a `.env` file in the project root with the following variables:

```env
DATABASE_URL=postgres://your_user:your_password@localhost:5432/taskforge
JWT_SECRET=a_very_secure_and_random_secret_key
# Optional: For configuring the server binding address
# SERVER_ADDRESS=127.0.0.1:8080
```

- `DATABASE_URL`: Connection string for your PostgreSQL database.
- `JWT_SECRET`: A strong, random secret key used for signing and verifying JWTs.
- `SERVER_ADDRESS` (Optional): The address and port for the server to listen on. Defaults to `127.0.0.1:8080` if not set and not overridden in `main.rs`.

## Database Setup

1.  Ensure your PostgreSQL server is running.
2.  Create a PostgreSQL database (if it doesn't exist already):
    ```sql
    CREATE DATABASE taskforge;
    -- Or use: createdb taskforge
    ```
3.  Set your `DATABASE_URL` in the `.env` file.
4.  Run the database migrations:
    ```bash
    cargo sqlx migrate run
    ```
5.  Prepare SQLx query data (generates `.sqlx` directory, which should be committed):
    ```bash
    cargo sqlx prepare
    ```

## Running the Application

1.  Ensure all prerequisites are installed and the `.env` file is configured.
2.  Build the project:
    ```bash
    cargo build
    ```
3.  Run the server:
    ```bash
    cargo run
    ```

The server will typically start at `http://127.0.0.1:8080`. If you've set `SERVER_ADDRESS` in your `.env` file *and* modified `main.rs` to use it, it will bind to that address.

## Development with Auto-Reload

For a smoother development experience, use `cargo-watch`:

1.  Install `cargo-watch` (if not already installed):
    ```bash
    cargo install cargo-watch
    ```
2.  Run the server with auto-reload:
    ```bash
    cargo watch -x run
    ```

## API Endpoints

All API routes are prefixed with `/api`.

> For a detailed description of all planned API endpoints, including those for future features, please see the [API Design Document](docs/api/API.md).

### Authentication

-   `POST /api/auth/register`: Register a new user.
    -   Request Body: `{"username": "user", "email": "user@example.com", "password": "securepassword"}`
    -   Response: `201 Created` with `{"token": "jwt_token", "user_id": 1}`
-   `POST /api/auth/login`: Login an existing user.
    -   Request Body: `{"email": "user@example.com", "password": "securepassword"}`
    -   Response: `200 OK` with `{"token": "jwt_token", "user_id": 1}`

### Tasks (Requires Authentication - Bearer Token)

-   `GET /api/tasks`: List tasks for the authenticated user. Supports query parameters:
    -   `status` (e.g., `todo`, `in_progress`, `done`)
    -   `priority` (e.g., `low`, `medium`, `high`, `urgent`)
    -   `assigned_to` (user ID)
    -   `search` (string for title/description)
-   `POST /api/tasks`: Create a new task.
    -   Request Body: `{"title": "New Task", "description": "Details", "priority": "medium", "status": "todo", "due_date": "2024-12-31T23:59:59Z"}`
-   `GET /api/tasks/{id}`: Get a specific task by its UUID.
-   `PUT /api/tasks/{id}`: Update a specific task by its UUID.
    -   Request Body: (Similar to POST, fields to update)
-   `DELETE /api/tasks/{id}`: Delete a specific task by its UUID.

### Health Check

-   `GET /health`: Check API health status. (No `/api` prefix for this route)

## Testing

Run the tests (unit and integration):

```bash
cargo test
```

## Project Structure

```
.env                       # Environment variables (ignored by git)
.gitignore
.sqlx/                     # SQLx query metadata (commit this)
Cargo.toml
Cargo.lock                 # Dependency lock file (commit this)
migrations/
├── YYYYMMDDHHMMSS_create_users_table.sql
├── YYYYMMDDHHMMSS_create_tasks_table.sql
└── ...                    # Other migration files
src/
├── main.rs                # Application entry point, HTTP server setup
├── lib.rs                 # Library root, module declarations
├── auth/
│   ├── mod.rs             # Authentication DTOs, get_user_id, re-exports
│   ├── middleware.rs      # Auth middleware logic
│   ├── password.rs        # Password hashing and verification
│   └── token.rs           # JWT generation and verification
├── error.rs               # Custom error types and handling
├── models/
│   ├── mod.rs             # Model re-exports
│   ├── task.rs            # Task struct, TaskInput, TaskQuery, TaskStatus, TaskPriority enums
│   └── user.rs            # User struct, UserInput
└── routes/
    ├── mod.rs             # Route configuration (config function)
    ├── auth.rs            # Authentication route handlers (login, register)
    ├── health.rs          # Health check route handler
    └── tasks.rs           # Task CRUD route handlers
tests/
├── auth.rs                # Integration tests for authentication flow
└── tasks.rs               # Integration tests for task CRUD operations
README.md
LICENSE
```

## Contributing

1.  Fork the repository.
2.  Create your feature branch (`git checkout -b feature/your-amazing-feature`).
3.  Make your changes, ensuring code quality and tests.
4.  Commit your changes (`git commit -m 'Add some amazing feature'`).
5.  Push to the branch (`git push origin feature/your-amazing-feature`).
6.  Open a Pull Request against the `main` branch.

## License

This project is licensed under the MIT License - see the `LICENSE` file for details. 