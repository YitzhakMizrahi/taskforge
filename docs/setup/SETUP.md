# Development Environment Setup

> **Note:** For a quick start, see the main `README.md`. This document provides more detailed setup instructions.

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
# Create database (if it doesn't exist)
createdb taskforge

# Ensure DATABASE_URL in .env points to this database
# Then, run migrations:
sqlx migrate run

# Prepare SQLx query data (if not already done or if queries change):
cargo sqlx prepare
```

### 4. Environment Configuration
Create a `.env` file in the project root (refer to `README.md` for the latest recommended variables):
```env
DATABASE_URL=postgres://your_user:your_password@localhost/taskforge
# REDIS_URL=redis://localhost:6379 # Redis is optional and not yet integrated
JWT_SECRET=your_very_strong_secret_key_here
RUST_LOG=info,taskforge=debug # Example: info level for all, debug for taskforge crate
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
docker build -t taskforge .
```

### 2. Run Container
```bash
docker run -p 8080:8080 taskforge
```

## Development Tools

### Recommended VS Code Extensions
- `rust-analyzer` (Essential for Rust development)
- `CodeLLDB` (For debugging)
- `crates` (Helps manage dependencies in Cargo.toml)
- `Even Better TOML` (Improved TOML file support)
- `SQLx Linter` (Provides SQL linting if you write raw SQL in strings, complements `sqlx prepare`)
- `EditorConfig for VS Code` (If using an .editorconfig file)
- `GitLens` (Enhanced Git capabilities)

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
   - Ensure database `taskforge` exists

2. Build errors
   - Run `cargo clean`
   - Update dependencies
   - Check Rust version

3. Runtime errors
   - Check logs (consider `RUST_LOG` setting in `.env`)
   - Verify environment variables
   - Check database migrations

## Additional Resources
- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/docs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) 