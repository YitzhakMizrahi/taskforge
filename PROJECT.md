# TaskForge

## Project Overview
A high-performance, real-time collaborative task management system built with Rust, demonstrating the language's strengths in concurrent operations and web development.

## Project Status
🚧 In Development

## Documentation
- [Architecture](docs/architecture/ARCHITECTURE.md) - System design and architecture
- [API Documentation](docs/api/API.md) - API endpoints and interfaces
- [Setup Guide](docs/setup/SETUP.md) - Development environment setup
- [Contributing](docs/contributing/CONTRIBUTING.md) - How to contribute
- [Development Rules](.cursor/rules/cursor-rules.mdc) - Development guidelines

## Objectives
- [ ] Create a robust, scalable task management system
- [ ] Implement real-time updates using WebSocket
- [ ] Build a secure authentication system
- [ ] Develop an efficient database schema
- [ ] Create a comprehensive API
- [ ] Implement file handling capabilities
- [ ] Add real-time notifications
- [ ] Ensure high performance under load

## Technical Stack
- **Backend Framework**: Actix-web
- **Async Runtime**: Tokio
- **Database**: PostgreSQL
- **ORM**: SQLx
- **Authentication**: JWT
- **Real-time**: WebSocket
- **Serialization**: Serde
- **Caching**: Redis (optional)

## Development Phases

### Phase 1: Foundation
- [ ] Project setup and structure
- [ ] Basic server configuration
- [ ] Database setup
- [ ] Basic API structure

### Phase 2: Core Features
- [ ] User authentication
- [ ] Task CRUD operations
- [ ] Basic real-time updates
- [ ] Team management

### Phase 3: Advanced Features
- [ ] File attachments
- [ ] Real-time notifications
- [ ] Search and filtering
- [ ] Activity logging

### Phase 4: Optimization
- [ ] Performance testing
- [ ] Caching implementation
- [ ] Load testing
- [ ] Security audit

## Current Focus
- Setting up project structure
- Implementing basic server functionality

## Next Steps
1. Initialize project with Cargo
2. Set up development environment
3. Create basic server structure
4. Implement database models

## Development Guidelines
- Follow rules in `.cursor/rules/cursor-rules.mdc`
- Maintain comprehensive documentation
- Write tests for all new features
- Review and update this document regularly

## Notes and Decisions
- Project named "TaskForge" to reflect its purpose of building and managing tasks
- Document important decisions here
- Track technical challenges and solutions
- Note any architecture changes

## Resources
- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/docs/)
- [Tokio Documentation](https://tokio.rs/docs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)

## Updates Log
- Initial project documentation created
- Defined project structure and objectives
- Established development guidelines
- Created comprehensive documentation structure
- Project renamed to TaskForge 