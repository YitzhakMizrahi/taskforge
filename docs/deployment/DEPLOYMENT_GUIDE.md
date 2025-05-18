# TaskForge Deployment Guide

This document provides guidance and considerations for deploying the TaskForge application to a production environment. While TaskForge is under active development, these notes aim to ensure a smoother transition to production when the time comes.

## 1. Security

### 1.1. HTTPS (TLS/SSL)
- **Recommendation:** Always run TaskForge behind a reverse proxy (e.g., Nginx, Caddy, HAProxy, or a cloud provider's load balancer) that handles TLS termination. This ensures all traffic between clients and the application is encrypted (HTTPS).
- Do not expose the Actix Web server directly to the internet without TLS.

### 1.2. CORS (Cross-Origin Resource Sharing)
- **Development:** The application is configured with `Cors::default().allow_any_origin()` for ease of development.
- **Production:** This **MUST** be changed. Configure CORS to allow requests only from the specific domain(s) where your frontend application is hosted. Allowing any origin in production is a security risk.

### 1.3. Secret Management
- **Environment Variables:** TaskForge uses environment variables for sensitive information:
    - `DATABASE_URL`: Connection string for the PostgreSQL database.
    - `JWT_SECRET`: A strong, unique secret key for signing and verifying JSON Web Tokens.
    - `SERVER_ADDRESS`: The address and port the server binds to (e.g., `0.0.0.0:8080`).
- **Production:**
    - **NEVER** hardcode these secrets into your deployment scripts or source code.
    - Use secure methods for managing these secrets, such as:
        - Environment variables injected by your deployment platform (e.g., Docker Compose environment files, Kubernetes Secrets, PaaS environment variable settings).
        - A dedicated secret management service (e.g., HashiCorp Vault, AWS Secrets Manager, Google Cloud Secret Manager, Azure Key Vault).
    - Ensure `JWT_SECRET` is cryptographically strong and kept confidential. Rotate it if compromised.

## 2. Database
### 2.1. PostgreSQL
- Ensure your PostgreSQL instance is appropriately secured (strong passwords, network access controls).
- Implement a regular backup strategy for your production database.
- Monitor database performance and resource usage.

### 2.2. Connection Pooling
- TaskForge utilizes `sqlx::PgPool` for database connection pooling. Ensure the pool size is appropriately configured for your expected load. The default `sqlx` pool size is 10. This can be configured via the `DATABASE_URL` (e.g., `postgres://user:pass@host/db?pool_max_connections=20`).

### 2.3. Migrations
- Database migrations are managed by `sqlx-cli`. Always run migrations as part of your deployment process before starting the new application version.
  ```bash
  cargo sqlx migrate run
  ```

## 3. Application Configuration
### 3.1. Server Address
- The `SERVER_ADDRESS` environment variable (e.g., `0.0.0.0:8080`) controls where the Actix Web server listens. In many production setups, especially with reverse proxies or containers, `0.0.0.0` is appropriate to listen on all available network interfaces within the container/VM. The reverse proxy then handles external access.

### 3.2. Logging
- **Development:** Uses `actix_web::middleware::Logger::default()`.
- **Production:**
    - Consider configuring the log format for better machine readability if you're using a centralized logging system (e.g., ELK stack, Splunk, Datadog). Actix's logger can be customized.
    - You might want to adjust log levels based on the environment (e.g., `INFO` or `WARN` in production, `DEBUG` in development). The `RUST_LOG` environment variable is commonly used for this with crates like `env_logger` or `tracing-subscriber`. TaskForge currently uses the default `Logger`.
    - Ensure logs are stored appropriately and rotated to prevent disk space issues.

## 4. Building for Production
- Compile your Rust application in release mode for optimal performance:
  ```bash
  cargo build --release
  ```
- The resulting binary will be in `target/release/taskforge`.

## 5. Containerization (Optional but Recommended)
- Consider deploying TaskForge as a Docker container for consistency and portability.
- A `Dockerfile` would typically:
    - Start from a Rust base image.
    - Copy `Cargo.toml`, `Cargo.lock`, and the `src` directory.
    - Build the application in release mode.
    - Create a smaller final image, copying only the compiled binary from the builder stage.
    - Expose the application port and set the `CMD` to run the binary.

## 6. Health Checks
- The `/health` endpoint can be used by your deployment platform or load balancer to verify the application is running and healthy.

---

This guide is a starting point. Always adapt deployment strategies to your specific infrastructure and security requirements. 