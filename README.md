# TaskForge API

A RESTful API for task management built with Rust and Actix Web.

## Features

- User authentication with JWT tokens
- Task management (CRUD operations)
- Input validation
- Error handling
- Database integration with PostgreSQL
- CORS support
- Environment-based configuration

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Cargo

## Environment Variables

Create a `.env` file in the project root with the following variables:

```env
DATABASE_URL=postgres://username:password@localhost:5432/taskforge
JWT_SECRET=your-secret-key
SERVER_PORT=8080
SERVER_HOST=127.0.0.1
CORS_ORIGINS=http://localhost:3000,http://localhost:8080
```

## Database Setup

1. Create a PostgreSQL database:
```sql
CREATE DATABASE taskforge;
```

2. Run the migrations:
```bash
sqlx database create
sqlx migrate run
```

## Running the Application

1. Build the project:
```bash
cargo build
```

2. Run the server:
```bash
cargo run
```

The server will start at `http://localhost:8080` (or your configured host and port).

## Development with Auto-Reload

For a smoother development experience, you can use [cargo-watch](https://github.com/watchexec/cargo-watch) to automatically rebuild and restart the server whenever you change your code.

### Install cargo-watch (one-time):
```bash
cargo install cargo-watch
```

### Run the server with auto-reload:
```bash
cargo watch -x 'run'
```

This will watch your project for changes and restart the server automatically.

## API Endpoints

### Authentication

- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login and get JWT token

### Tasks

- `GET /api/tasks` - List all tasks
- `POST /api/tasks` - Create a new task
- `GET /api/tasks/{id}` - Get a specific task
- `PUT /api/tasks/{id}` - Update a task
- `DELETE /api/tasks/{id}` - Delete a task

### Health Check

- `GET /health` - Check API health status

## Testing

Run the tests with:

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs           # Application entry point
├── auth.rs           # Authentication middleware and utilities
├── config.rs         # Configuration management
├── errors.rs         # Error handling
├── models.rs         # Data models
└── routes/           # API routes
    ├── mod.rs        # Route configuration
    ├── auth.rs       # Authentication routes
    ├── health.rs     # Health check endpoint
    └── tasks.rs      # Task management routes
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 