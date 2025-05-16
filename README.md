# TaskForge

> Forge your tasks, forge your success

A high-performance, real-time collaborative task management system built with Rust, demonstrating the language's strengths in concurrent operations and web development.

## Features

- 🔄 Real-time task updates
- 👥 Team collaboration
- 🔐 Secure authentication
- 📁 File attachments
- 🔔 Real-time notifications
- 🚀 High performance
- 🔍 Advanced search
- 📊 Activity tracking

## Quick Start

### Prerequisites
- Rust (latest stable)
- PostgreSQL 14+
- Redis (optional)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/taskforge.git
cd taskforge
```

2. Set up the environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Install dependencies:
```bash
cargo build
```

4. Run the application:
```bash
cargo run
```

## Documentation

- [Project Documentation](PROJECT.md) - Detailed project information
- [Architecture](docs/architecture/ARCHITECTURE.md) - System design
- [API Documentation](docs/api/API.md) - API endpoints
- [Setup Guide](docs/setup/SETUP.md) - Development setup
- [Contributing](docs/contributing/CONTRIBUTING.md) - How to contribute

## Tech Stack

- **Backend**: Rust, Actix-web
- **Database**: PostgreSQL
- **Cache**: Redis
- **Real-time**: WebSocket
- **Authentication**: JWT

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Run with hot reload
cargo watch -x run
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](docs/contributing/CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Actix-web](https://actix.rs/)
- [Tokio](https://tokio.rs/)
- [SQLx](https://github.com/launchbadge/sqlx)
- All our contributors 