# Development Environment Setup

## Prerequisites
- Rust (latest stable version)
- PostgreSQL 14+
- Redis (optional, for caching)
- Docker (optional, for containerization)

## Local Development Setup

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Dependencies
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql
```

### 3. Database Setup
```bash
# Create database
createdb task_manager

# Run migrations
sqlx database create
sqlx migrate run
```

### 4. Environment Configuration
Create a `.env` file in the project root:
```env
DATABASE_URL=postgres://postgres:postgres@localhost/task_manager
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
RUST_LOG=debug
```

### 5. Build and Run
```bash
# Build
cargo build

# Run
cargo run

# Run tests
cargo test

# Run with hot reload
cargo watch -x run
```

## Docker Setup

### 1. Build Image
```bash
docker build -t task-manager .
```

### 2. Run Container
```bash
docker run -p 8080:8080 task-manager
```

## Development Tools

### Recommended VS Code Extensions
- rust-analyzer
- CodeLLDB
- Better TOML
- SQLx

### Useful Commands
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open

# Check for updates
cargo update
```

## Troubleshooting

### Common Issues
1. Database connection errors
   - Check PostgreSQL is running
   - Verify DATABASE_URL in .env
   - Ensure database exists

2. Build errors
   - Run `cargo clean`
   - Update dependencies
   - Check Rust version

3. Runtime errors
   - Check logs
   - Verify environment variables
   - Check database migrations

## Additional Resources
- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/docs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) 