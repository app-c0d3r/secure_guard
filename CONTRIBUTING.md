# Contributing to SecureGuard

Thank you for your interest in contributing to SecureGuard! This document provides guidelines for contributing to the project.

## 📋 Documentation Standards

### Creating New Documentation

All documentation should be placed in the [`docs/`](docs/) directory and follow these guidelines:

#### File Naming Convention
- Use descriptive names with underscores: `Feature_Implementation_Guide.md`
- Include version/date for specifications: `API_Specification_v2.0.md`
- Use prefixes for organization:
  - `Setup_` - Environment and installation guides
  - `API_` - API documentation and specifications
  - `Architecture_` - System design and architecture
  - `User_` - End-user guides and tutorials
  - `Dev_` - Development process and guidelines

#### Required Document Structure
```markdown
# Document Title

**Document Version:** X.X  
**Last Updated:** YYYY-MM-DD  
**Status:** Draft/Ready/Archived  
**Author:** Name or Team

## Overview
Brief description of the document's purpose

## Table of Contents
- [Section 1](#section-1)
- [Section 2](#section-2)

## Content
Main documentation content

## Next Steps
What comes after this document

---
**Next Review:** Date when this should be reviewed again
```

### Documentation Types

#### Setup and Installation Guides
- Clear step-by-step instructions
- Prerequisites clearly listed
- Troubleshooting sections
- Platform-specific notes (Windows/Linux/macOS)

#### Architecture Documents
- System overview diagrams
- Component interactions
- Data flow descriptions
- Security considerations

#### API Documentation
- Endpoint specifications
- Request/response examples
- Authentication requirements
- Error handling

#### Development Guides
- Code standards and conventions
- Testing requirements
- Build and deployment processes
- Development workflow

## 🔧 Code Contribution Guidelines

### Rust Code Standards

#### Code Formatting
```bash
# Format code before committing
cargo fmt

# Run clippy for linting
cargo clippy -- -D warnings

# Run security audit
cargo audit
```

#### Naming Conventions
- **Types**: `PascalCase` (e.g., `SecurityEvent`, `ThreatAlert`)
- **Functions/Variables**: `snake_case` (e.g., `create_user`, `agent_id`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRY_ATTEMPTS`)
- **Modules**: `snake_case` (e.g., `threat_service`, `auth_handler`)

#### Error Handling
- Use `Result<T, SecureGuardError>` for fallible operations
- Use `thiserror` for custom error types
- Use `anyhow` for error context in applications
- Never use `unwrap()` or `expect()` in production code

#### Testing Requirements
- **Unit Tests**: All business logic functions
- **Integration Tests**: API endpoints and database operations
- **Coverage**: Minimum 80% code coverage
- **Documentation Tests**: All public APIs must have doc tests

```rust
/// Example function with proper documentation
/// 
/// # Examples
/// 
/// ```
/// use secureguard_shared::User;
/// let user = create_test_user("test@example.com");
/// assert_eq!(user.email, "test@example.com");
/// ```
pub fn create_test_user(email: &str) -> User {
    // Implementation
}
```

### Database Guidelines

#### Migrations
- Use descriptive migration names: `V003_add_user_roles_table.sql`
- Include rollback instructions in comments
- Test migrations on sample data
- Never modify existing migrations after merge

#### Schema Design
- Use UUID primary keys for all entities
- Include `created_at` and `updated_at` timestamps
- Use JSONB for flexible data storage
- Implement proper indexes for query performance

### API Design Standards

#### RESTful Endpoints
```
GET    /api/v1/users              # List users
POST   /api/v1/users              # Create user
GET    /api/v1/users/{id}         # Get specific user
PUT    /api/v1/users/{id}         # Update user
DELETE /api/v1/users/{id}         # Delete user
```

#### Response Format
```json
{
  "data": { ... },
  "meta": {
    "timestamp": "2025-08-18T10:00:00Z",
    "version": "1.0"
  }
}
```

#### Error Response Format
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input provided",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    }
  }
}
```

## 🔒 Security Requirements

### Code Security
- Never commit secrets, API keys, or passwords
- Use environment variables for configuration
- Validate all user inputs
- Implement proper authentication and authorization
- Use parameterized queries to prevent SQL injection

### Documentation Security
- Avoid including sensitive information in docs
- Use placeholder values in examples
- Mark sensitive sections clearly
- Regular security review of documentation

## 🧪 Testing Standards

### Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_creation_success() {
        // Arrange
        let test_db = TestDatabase::new().await;
        
        // Act
        let result = create_user(&test_db, valid_request()).await;
        
        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }
}
```

### Integration Tests
- Test complete workflows
- Use test database for isolation
- Clean up test data after each test
- Test error conditions and edge cases

## 📝 Pull Request Process

### Before Submitting
1. **Code Quality**
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   cargo audit
   ```

2. **Documentation**
   - Update relevant documentation
   - Add/update API docs for new endpoints
   - Include examples for new features

3. **Testing**
   - Add tests for new functionality
   - Ensure all existing tests pass
   - Test edge cases and error conditions

### PR Description Template
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
```

## 🔄 Review Process

### Code Review Checklist
- [ ] Code follows established patterns
- [ ] Security best practices followed
- [ ] Tests are comprehensive
- [ ] Documentation is updated
- [ ] Performance considerations addressed

### Documentation Review
- [ ] Accuracy of technical content
- [ ] Clarity and readability
- [ ] Completeness of information
- [ ] Proper formatting and structure

## 🚀 Release Process

### Version Management
- Follow [Semantic Versioning](https://semver.org/)
- Update version in `Cargo.toml` files
- Tag releases in git
- Update CHANGELOG.md

### Documentation Updates
- Review and update all documentation for releases
- Archive outdated documentation
- Update API version references
- Publish updated documentation

---

## 📞 Getting Help

- **Issues**: Create GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Security**: Report security issues privately to the maintainers

Thank you for contributing to SecureGuard! 🛡️