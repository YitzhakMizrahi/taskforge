# Contributing Guidelines

## Code of Conduct
- Be respectful and inclusive
- Be patient and welcoming
- Be thoughtful
- Be collaborative
- When disagreeing, try to understand why

## Development Process

### 1. Setting Up
1. Fork the repository
2. Clone your fork
3. Set up development environment (see `docs/setup/SETUP.md`)
4. Create a new branch for your feature

### 2. Making Changes
1. Follow the coding standards in `.cursor/rules/cursor-rules.mdc`
2. Write tests for new features
3. Update documentation
4. Keep commits atomic and focused

### 3. Testing
- Run all tests: `cargo test`
- Run specific tests: `cargo test <test_name>`
- Check formatting: `cargo fmt`
- Run linter: `cargo clippy`

### 4. Documentation
- Update relevant documentation
- Add comments for complex logic
- Update API documentation if needed
- Keep README.md up to date

### 5. Submitting Changes
1. Push your changes to your fork
2. Create a pull request
3. Fill out the PR template
4. Wait for review

## Pull Request Process

### PR Template
```markdown
## Description
[Describe your changes]

## Related Issues
[Link to related issues]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Documentation
- [ ] README.md updated
- [ ] API documentation updated
- [ ] Code comments added/updated
```

### Review Process
1. Automated checks must pass
2. At least one review required
3. All comments must be addressed
4. Maintainer approval required

## Style Guide
- Follow Rust's official style guide
- Use `rustfmt` for formatting
- Follow naming conventions
- Write clear commit messages

## Commit Messages
Format:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation
- style: Formatting
- refactor: Code restructuring
- test: Adding tests
- chore: Maintenance

## Getting Help
- Check existing documentation
- Search existing issues
- Ask in discussions
- Contact maintainers

## Recognition
- Contributors will be listed in README.md
- Significant contributions will be specially acknowledged
- All contributors will be added to the repository's contributors list 