# TaskForge

[![CI](https://github.com/YitzhakMizrahi/taskforge/actions/workflows/ci.yml/badge.svg)](https://github.com/YitzhakMizrahi/taskforge/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75.0+-blue.svg)](https://www.rust-lang.org)
[![Code style: rustfmt](https://img.shields.io/badge/code%20style-rustfmt-000000.svg)](https://github.com/rust-lang/rustfmt)

A high-performance, real-time collaborative task management system built with Rust.

## Features

- Real-time task updates using WebSocket
- Secure authentication system
- Efficient database operations
- RESTful API
- File handling capabilities
- Real-time notifications
- High performance under load

## Tech Stack

- **Backend Framework**: Actix-web
- **Async Runtime**: Tokio
- **Database**: PostgreSQL
- **ORM**: SQLx
- **Authentication**: JWT
- **Real-time**: WebSocket
- **Serialization**: Serde
- **Caching**: Redis (optional)

## Getting Started

### Prerequisites

- Rust 1.75.0 or later
- PostgreSQL
- Redis (optional)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/YitzhakMizrahi/taskforge.git
   cd taskforge
   ```

2. Set up the development environment:
   ```bash
   ./scripts/setup_dev.sh
   ```

3. Configure environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. Run the development server:
   ```bash
   cargo run
   ```

## Development

### Common Tasks

```bash
# Run tests
make test

# Format code
make fmt

# Run linter
make lint

# Watch for changes
make watch

# Full development workflow
make dev
```

### Project Structure

```
taskforge/
├── src/              # Source code
├── tests/            # Test files
├── docs/             # Documentation
│   ├── api/         # API documentation
│   ├── architecture/# Architecture documentation
│   ├── setup/       # Setup guides
│   └── contributing/# Contributing guidelines
└── scripts/         # Development scripts
```

## Contributing

Please read [CONTRIBUTING.md](docs/contributing/CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Actix-web](https://actix.rs/)
- [Tokio](https://tokio.rs/)
- [SQLx](https://github.com/launchbadge/sqlx) 